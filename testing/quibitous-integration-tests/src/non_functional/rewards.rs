use crate::startup;
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::testing::{benchmark_consumption, benchmark_endurance, ResourcesUsage};
use quibitous_automation::{qcli::QCli, quibitous::ConfigurationBuilder};
use quibitous_lib::interfaces::ActiveSlotCoefficient;
use quibitestkit::process as process_utils;
use std::time::Duration;
use silica::TransactionHash;

#[test]
pub fn collect_reward_for_15_minutes() {
    let qcli: QCli = Default::default();
    let duration_48_hours = Duration::from_secs(900);

    let mut sender = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let stake_pool_owners = [
        sender.clone(),
        receiver.clone(),
        silica::Wallet::default(),
        silica::Wallet::default(),
        silica::Wallet::default(),
        silica::Wallet::default(),
        silica::Wallet::default(),
        silica::Wallet::default(),
    ];
    let (quibitous, _stake_pool_ids) = startup::start_stake_pool(
        &stake_pool_owners,
        &[],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(20)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_slot_duration(3),
    )
    .unwrap();

    let benchmark_endurance = benchmark_endurance("collect_reward_for_15_minutes")
        .target(duration_48_hours)
        .start();

    let mut benchmark_consumption =
        benchmark_consumption("collect_reward_for_15_minutes_resources")
            .target(ResourcesUsage::new(10, 200_000, 5_000_000))
            .for_process("Node 15 minutes up", quibitous.pid() as usize)
            .start();

    loop {
        let new_transaction = silica::FragmentBuilder::new(
            &quibitous.genesis_block_hash(),
            &quibitous.fees(),
            BlockDate::first().next_epoch(),
        )
        .transaction(&sender, receiver.address(), 10.into())
        .unwrap()
        .encode();

        qcli.rest()
            .v0()
            .message()
            .post(&new_transaction, quibitous.rest_uri());
        sender.confirm_transaction();

        benchmark_consumption.snapshot().unwrap();

        if benchmark_endurance.max_endurance_reached() {
            benchmark_consumption.stop().print();
            benchmark_endurance.stop().print();
            return;
        }

        if let Err(err) = quibitous.check_no_errors_in_log() {
            let message = format!("{}", err);
            benchmark_endurance.exception(message.clone()).print();
            benchmark_consumption.exception(message.clone()).print();
            std::panic::panic_any(message);
        }

        benchmark_consumption.snapshot().unwrap();
        if benchmark_endurance.max_endurance_reached() {
            benchmark_consumption.stop().print();
            benchmark_endurance.stop().print();
            return;
        }
        process_utils::sleep(5);

        let _rewards = quibitous
            .rest()
            .reward_history(1)
            .expect("failed to get last reward");
    }
}
