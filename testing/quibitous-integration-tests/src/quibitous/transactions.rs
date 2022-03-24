use crate::startup;
use chain_impl_mockchain::{block::BlockDate, fee::LinearFee};
use quibitous_automation::testing::time::wait_for_epoch;
use quibitous_automation::{qcli::JCli, quibitous::ConfigurationBuilder};
use quibitous_lib::interfaces::{ActiveSlotCoefficient, Mempool, Value};
use silica::TransactionHash;

#[test]
pub fn accounts_funds_are_updated_after_transaction() {
    let qcli: JCli = Default::default();
    let receiver = silica::Wallet::default();
    let mut sender = silica::Wallet::default();
    let fee = LinearFee::new(1, 1, 1);
    let value_to_send = 1;

    let (quibitous, _) = startup::start_stake_pool(
        &[sender.clone()],
        &[receiver.clone()],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(20)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_slot_duration(3)
            .with_linear_fees(fee)
            .with_mempool(Mempool {
                pool_max_entries: 1_000_000usize.into(),
                log_max_entries: 1_000_000usize.into(),
                persistent_log: None,
            }),
    )
    .unwrap();

    let sender_account_state_before = qcli
        .rest()
        .v0()
        .account_stats(sender.address().to_string(), quibitous.rest_uri());
    let receiever_account_state_before = qcli
        .rest()
        .v0()
        .account_stats(&receiver.address().to_string(), &quibitous.rest_uri());

    let sender_value_before = sender_account_state_before.value();
    let receiver_value_before = receiever_account_state_before.value();

    let new_transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        BlockDate::first().next_epoch(),
    )
    .transaction(&sender, receiver.address(), value_to_send.into())
    .unwrap()
    .encode();

    qcli.fragment_sender(&quibitous)
        .send(&new_transaction)
        .assert_in_block();

    sender.confirm_transaction();

    let sender_account_state = qcli
        .rest()
        .v0()
        .account_stats(sender.address().to_string(), &quibitous.rest_uri());
    let receiver_account_state = qcli
        .rest()
        .v0()
        .account_stats(receiver.address().to_string(), &quibitous.rest_uri());

    let sender_value_before_u64: u64 = (*sender_value_before).into();
    let receiver_value_before_u64: u64 = (*receiver_value_before).into();

    let sender_last_reward: u64 = (*sender_account_state.last_rewards().reward()).into();

    let sender_expected_value: Value =
        (sender_value_before_u64 - value_to_send - fee.constant - (fee.coefficient * 2)
            + sender_last_reward)
            .into();
    let receiver_expected_value: Value = (receiver_value_before_u64 + value_to_send).into();

    let sender_account_state_value: Value = *sender_account_state.value();
    let receiver_account_state_value: Value = *receiver_account_state.value();

    assert_eq!(
        sender_expected_value, sender_account_state_value,
        "sender value after transaction"
    );
    assert_eq!(
        receiver_expected_value, receiver_account_state_value,
        "receiver value after transaction"
    );
}

#[test]
fn expired_transactions_rejected() {
    let receiver = silica::Wallet::default();
    let sender = silica::Wallet::default();

    let (quibitous, _) = startup::start_stake_pool(
        &[sender.clone()],
        &[receiver.clone()],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(30)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_linear_fees(LinearFee::new(0, 0, 0))
            .with_mempool(Mempool {
                pool_max_entries: 1_000.into(),
                log_max_entries: 1_000.into(),
                persistent_log: None,
            }),
    )
    .unwrap();

    let qcli = JCli::default();

    let valid_transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        chain_impl_mockchain::block::BlockDate::first().next_epoch(),
    )
    .transaction(&sender, receiver.address(), 100.into())
    .unwrap()
    .encode();

    qcli.fragment_sender(&quibitous)
        .send(&valid_transaction)
        .assert_in_block();

    wait_for_epoch(2, quibitous.rest());

    let expired_transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        chain_impl_mockchain::block::BlockDate::first().next_epoch(),
    )
    .transaction(&sender, receiver.address(), 200.into())
    .unwrap()
    .encode();

    // The fragment is rejected before even entering the mempool so there's no fragment log for it.
    // We therefore check the fragment processing summary instead.
    qcli.fragment_sender(&quibitous)
        .send(&expired_transaction)
        .assert_rejected_summary();
}

#[test]
fn transactions_with_long_time_to_live_rejected() {
    const MAX_EXPIRY_EPOCHS: u8 = 5;

    let receiver = silica::Wallet::default();
    let sender = silica::Wallet::default();

    let (quibitous, _) = startup::start_stake_pool(
        &[sender.clone()],
        &[receiver.clone()],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(30)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_linear_fees(LinearFee::new(0, 0, 0))
            .with_mempool(Mempool {
                pool_max_entries: 1_000.into(),
                log_max_entries: 1_000.into(),
                persistent_log: None,
            })
            .with_tx_max_expiry_epochs(MAX_EXPIRY_EPOCHS),
    )
    .unwrap();

    let qcli = JCli::default();

    let valid_transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        chain_impl_mockchain::block::BlockDate {
            epoch: MAX_EXPIRY_EPOCHS as u32,
            slot_id: 0,
        },
    )
    .transaction(&sender, receiver.address(), 100.into())
    .unwrap()
    .encode();

    qcli.fragment_sender(&quibitous)
        .send(&valid_transaction)
        .assert_in_block();

    let expired_transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        chain_impl_mockchain::block::BlockDate {
            epoch: MAX_EXPIRY_EPOCHS as u32 + 1,
            slot_id: 0,
        },
    )
    .transaction(&sender, receiver.address(), 200.into())
    .unwrap()
    .encode();

    qcli.fragment_sender(&quibitous)
        .send(&expired_transaction)
        .assert_rejected_summary();
}
