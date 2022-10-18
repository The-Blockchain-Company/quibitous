use crate::startup;
use chain_impl_mockchain::{block::BlockDate, fee::LinearFee};
use quibitous_automation::{qcli::QCli, quibitous::ConfigurationBuilder, testing::time};
use quibitous_lib::{
    crypto::{account::Identifier as AccountIdentifier, hash::Hash},
    interfaces::{ActiveSlotCoefficient, Stake, StakeDistributionDto},
};
use std::str::FromStr;
use silica::TransactionHash;

#[test]
pub fn stake_distribution() {
    let qcli: QCli = Default::default();
    let sender = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let stake_pool_owner_1 = silica::Wallet::default();
    let fee = LinearFee::new(1, 1, 1);
    let (quibitous, stake_pools) = startup::start_stake_pool(
        &[stake_pool_owner_1.clone()],
        &[sender.clone(), receiver],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(20)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_rewards_history()
            .with_linear_fees(fee)
            .with_total_rewards_supply(1_000_000.into())
            .with_slot_duration(3),
    )
    .unwrap();

    assert!(
        quibitous.rest().stake_distribution_at(1).is_err(),
        "stake distribution for epoch in future should return error"
    );

    let transaction_fee: u64 = fee.constant + fee.coefficient * 2;
    let transaction_amount = 1_000;
    let initial_funds_per_account = 1_000_000_000;
    let stake_pool_id = Hash::from_str(&stake_pools.get(0).unwrap().id().to_string()).unwrap();

    assert_distribution(
        initial_funds_per_account * 2,
        0,
        (stake_pool_id, initial_funds_per_account),
        quibitous.rest().stake_distribution().unwrap(),
    );

    let transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        BlockDate::first().next_epoch(),
    )
    .transaction(
        &sender,
        stake_pool_owner_1.address(),
        transaction_amount.into(),
    )
    .unwrap()
    .encode();

    qcli.fragment_sender(&quibitous)
        .send(&transaction)
        .assert_in_block();

    time::wait_for_epoch(2, quibitous.rest());

    let identifier: AccountIdentifier = stake_pool_owner_1.identifier().into();
    let reward: u64 = (*quibitous
        .rest()
        .epoch_reward_history(1)
        .unwrap()
        .accounts()
        .get(&identifier)
        .unwrap())
    .into();

    qcli.rest().v0().account_stats(
        stake_pool_owner_1.address().to_string(),
        quibitous.rest_uri(),
    );

    time::wait_for_epoch(3, quibitous.rest());

    qcli.rest().v0().account_stats(
        stake_pool_owner_1.address().to_string(),
        quibitous.rest_uri(),
    );

    assert_distribution(
        initial_funds_per_account * 2 - transaction_fee - transaction_amount,
        0,
        (
            stake_pool_id,
            initial_funds_per_account + transaction_amount + reward,
        ),
        quibitous.rest().stake_distribution_at(3).unwrap(),
    );
}

fn assert_distribution(
    unassigned: u64,
    dangling: u64,
    pool_stake: (Hash, u64),
    stake_distribution_dto: StakeDistributionDto,
) {
    let stake_distribution = stake_distribution_dto.stake;
    assert_eq!(
        Stake::from(unassigned),
        stake_distribution.unassigned,
        "unassigned"
    );
    assert_eq!(
        Stake::from(dangling),
        stake_distribution.dangling,
        "dangling"
    );
    let stake_pool_stake: Stake = *stake_distribution
        .pools
        .iter()
        .find(|(key, _)| *key == pool_stake.0)
        .map(|(_, stake)| stake)
        .unwrap();
    assert_eq!(
        Stake::from(pool_stake.1),
        stake_pool_stake,
        "stake pool {} stake",
        pool_stake.0
    );
}
