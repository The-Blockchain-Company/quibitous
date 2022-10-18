use crate::startup;
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::{
    qcli::{FragmentsCheck, QCli},
    quibitous::ConfigurationBuilder,
    testing::time,
};
use quibitous_lib::interfaces::{
    ActiveSlotCoefficient, BlockDate as BlockDateDto, KesUpdateSpeed,
};
pub use jortestkit::{
    console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode},
    load::{self, ConfigurationBuilder as LoadConfigurationBuilder, Monitor},
    prelude::Wait,
};
use mfive::generators::{
    BatchFragmentGenerator, FragmentGenerator, FragmentStatusProvider, TransactionGenerator,
};
use std::time::Duration;
use silica::{BlockDateGenerator, FragmentSender, FragmentSenderSetup};

#[test]
pub fn fragment_load_test() {
    let faucet = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let (mut quibitous, _) = startup::start_stake_pool(
        &[faucet.clone()],
        &[receiver.clone()],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(30)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_slot_duration(4)
            .with_block_content_max_size(204800.into())
            .with_epoch_stability_depth(10)
            .with_kes_update_speed(KesUpdateSpeed::new(43200).unwrap()),
    )
    .unwrap();

    quibitous.steal_temp_dir().unwrap().into_persistent();
    let settings = quibitous.rest().settings().unwrap();

    let configuration = LoadConfigurationBuilder::duration(Duration::from_secs(60))
        .step_delay(Duration::from_millis(500))
        .monitor(Monitor::Standard(1000))
        .shutdown_grace_period(Duration::from_secs(1))
        .status_pace(Duration::from_secs(1))
        .build();

    let mut request_generator = FragmentGenerator::new(
        faucet,
        receiver,
        quibitous.to_remote(),
        60,
        30,
        30,
        30,
        FragmentSender::new(
            quibitous.genesis_block_hash(),
            quibitous.fees(),
            BlockDateGenerator::rolling(
                &settings,
                BlockDate {
                    epoch: 1,
                    slot_id: 0,
                },
                false,
            ),
            FragmentSenderSetup::no_verify(),
        ),
    );

    request_generator.prepare(BlockDateDto::new(2, 0));

    let qcli: QCli = Default::default();
    let fragment_check = FragmentsCheck::new(qcli, &quibitous);
    let wait = Wait::new(Duration::from_secs(1), 25);
    fragment_check.wait_until_all_processed(&wait).unwrap();

    time::wait_for_epoch(3, quibitous.rest());

    load::start_async(
        request_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        configuration,
        "Wallet backend load test",
    );
}

#[test]
pub fn fragment_batch_load_test() {
    let mut faucet = silica::Wallet::default();

    let (mut quibitous, _) = startup::start_stake_pool(
        &[faucet.clone()],
        &[],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(60)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_slot_duration(4)
            .with_epoch_stability_depth(10)
            .with_kes_update_speed(KesUpdateSpeed::new(43200).unwrap()),
    )
    .unwrap();

    quibitous.steal_temp_dir().unwrap().into_persistent();

    let configuration = LoadConfigurationBuilder::duration(Duration::from_secs(60))
        .thread_no(5)
        .step_delay(Duration::from_secs(1))
        .monitor(Monitor::Standard(100))
        .shutdown_grace_period(Duration::from_secs(3))
        .status_pace(Duration::from_secs(1))
        .build();

    let settings = quibitous.rest().settings().unwrap();

    let mut request_generator = BatchFragmentGenerator::new(
        FragmentSenderSetup::no_verify(),
        quibitous.to_remote(),
        quibitous.genesis_block_hash(),
        quibitous.fees(),
        BlockDateGenerator::rolling(
            &settings,
            BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            false,
        ),
        10,
    );
    request_generator.fill_from_faucet(&mut faucet);

    load::start_async(
        request_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        configuration,
        "Wallet backend load test",
    );
}

#[test]
pub fn transaction_load_test() {
    let mut faucet = silica::Wallet::default();

    let (mut quibitous, _) = startup::start_stake_pool(
        &[faucet.clone()],
        &[],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(60)
            .with_consensus_genesis_optimum_active_slot_coeff(ActiveSlotCoefficient::MAXIMUM)
            .with_slot_duration(4)
            .with_epoch_stability_depth(10)
            .with_kes_update_speed(KesUpdateSpeed::new(43200).unwrap()),
    )
    .unwrap();

    quibitous.steal_temp_dir().unwrap().into_persistent();
    let settings = quibitous.rest().settings().unwrap();

    let configuration = LoadConfigurationBuilder::duration(Duration::from_secs(60))
        .step_delay(Duration::from_millis(100))
        .monitor(Monitor::Standard(100))
        .shutdown_grace_period(Duration::from_millis(100))
        .status_pace(Duration::from_secs(1))
        .build();

    let mut request_generator = TransactionGenerator::new(
        FragmentSenderSetup::no_verify(),
        quibitous.to_remote(),
        quibitous.genesis_block_hash(),
        quibitous.fees(),
        BlockDateGenerator::rolling(
            &settings,
            BlockDate {
                epoch: 1,
                slot_id: 0,
            },
            false,
        ),
    );
    request_generator.fill_from_faucet(&mut faucet);

    load::start_async(
        request_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        configuration,
        "Wallet backend load test",
    );
}
