use std::time::Duration;

use crate::startup;
use assert_fs::{fixture::PathChild, TempDir};
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::quibitous::ConfigurationBuilder;
use quibitous_lib::interfaces::{Mempool, PersistentLog};
pub use quibitestkit::{
    console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode},
    load::{self, ConfigurationBuilder as LoadConfigurationBuilder, Monitor},
};
use mfive::generators::{BatchFragmentGenerator, FragmentStatusProvider};
use silica::{BlockDateGenerator, FragmentSenderSetup, PersistentLogViewer};

#[test]
pub fn persistent_log_load_test() {
    let mut faucet = silica::Wallet::default();

    let temp_dir = TempDir::new().unwrap();
    let persistent_log_path = temp_dir.child("fragment_dump");

    let quibitous = startup::start_bft(
        vec![&faucet],
        ConfigurationBuilder::new()
            .with_slots_per_epoch(60)
            .with_slot_duration(1)
            .with_mempool(Mempool {
                pool_max_entries: 1_000_000usize.into(),
                log_max_entries: 1_000_000usize.into(),
                persistent_log: Some(PersistentLog {
                    dir: persistent_log_path.path().to_path_buf(),
                }),
            }),
    )
    .unwrap();

    let batch_size = 10;
    let requests_per_thread = 50;
    let threads_count = 1;

    let configuration = LoadConfigurationBuilder::requests_per_thread(requests_per_thread)
        .thread_no(threads_count)
        .step_delay(Duration::from_secs(1))
        .monitor(Monitor::Standard(100))
        .shutdown_grace_period(Duration::from_secs(1))
        .status_pace(Duration::from_millis(30))
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
        batch_size,
    );
    request_generator.fill_from_faucet(&mut faucet);

    let base_fragment_count = quibitous.rest().fragment_logs().unwrap().len();

    load::start_async(
        request_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        configuration,
        "Wallet backend load test",
    );

    let persistent_log_viewer = PersistentLogViewer::new(persistent_log_path.path().to_path_buf());
    assert_eq!(
        base_fragment_count
            + (batch_size as usize) * (requests_per_thread as usize) * threads_count,
        persistent_log_viewer.get_all().len()
    );
}
