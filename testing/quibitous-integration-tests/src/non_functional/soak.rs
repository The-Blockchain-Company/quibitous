use crate::startup;
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::testing::{benchmark_consumption, benchmark_endurance};
use quibitous_automation::{qcli::QCli, quibitous::ConfigurationBuilder};
use quibitous_lib::interfaces::{ActiveSlotCoefficient, KesUpdateSpeed, Mempool};
use quibitestkit::process::Wait;
use std::time::Duration;
use silica::TransactionHash;

#[test]
pub fn test_blocks_are_being_created_for_7_hours() {
    let qcli: QCli = Default::default();
    let duration_48_hours = Duration::from_secs(25_200);

    let mut receiver = silica::Wallet::default();
    let mut sender = silica::Wallet::default();
    let (quibitous, _) = startup::start_stake_pool(
        &[sender.clone()],
        &[],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(20)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_slot_duration(3)
            .with_kes_update_speed(KesUpdateSpeed::new(43200).unwrap())
            .with_mempool(Mempool {
                pool_max_entries: 1_000_000usize.into(),
                log_max_entries: 1_000_000usize.into(),
                persistent_log: None,
            }),
    )
    .unwrap();

    let benchmark_endurance = benchmark_endurance("test_blocks_are_being_created_for_48_hours")
        .target(duration_48_hours)
        .start();

    let mut benchmark_consumption =
        benchmark_consumption("test_blocks_are_being_created_for_48_hours_resources")
            .bare_metal_stake_pool_consumption_target()
            .for_process("Node 48 hours up", quibitous.pid() as usize)
            .start();

    loop {
        let new_transaction = silica::FragmentBuilder::new(
            &quibitous.genesis_block_hash(),
            &quibitous.fees(),
            BlockDate::first().next_epoch(),
        )
        .transaction(&sender, receiver.address(), 1.into())
        .unwrap()
        .encode();

        let wait: Wait = Wait::new(Duration::from_secs(10), 10);

        let checker = qcli.fragment_sender(&quibitous).send(&new_transaction);
        let fragment_id = checker.fragment_id();
        match checker.wait_until_processed(&wait) {
            Ok(fragment_id) => fragment_id,
            Err(err) => {
                let message = format!("error: {}, transaction with id: {} was not in a block as expected. Message log: {:?}. Quibitous log: {}",
                            err,
                            fragment_id,
                            qcli.rest().v0().message().logs(quibitous.rest_uri()),
                            quibitous.logger.get_log_content()
                        );
                benchmark_endurance.exception(message.clone()).print();
                benchmark_consumption.exception(message.clone()).print();
                std::panic::panic_any(message);
            }
        };
        sender.confirm_transaction();

        benchmark_consumption.snapshot().unwrap();

        if benchmark_endurance.max_endurance_reached() {
            benchmark_consumption.stop().print();
            benchmark_endurance.stop().print();
            return;
        }

        std::mem::swap(&mut sender, &mut receiver);
    }
}
