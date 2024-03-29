use crate::startup;
use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::qcli::QCli;
use quibitous_automation::quibitous::ConfigurationBuilder;
use quibitous_lib::interfaces::{Mempool, PersistentLog};
pub use quibitestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use silica::{PersistentLogViewer, TransactionHash};

#[test]
/// Verifies that no log entries are created for fragments that are already expired when received.
fn rejected_fragments_have_no_log() {
    let receiver = silica::Wallet::default();
    let sender = silica::Wallet::default();

    let log_path = TempDir::new().unwrap().child("log_path");

    let (quibitous, _) = startup::start_stake_pool(
        &[sender.clone()],
        &[receiver.clone()],
        ConfigurationBuilder::new().with_mempool(Mempool {
            pool_max_entries: 1_000.into(),
            log_max_entries: 1_000.into(),
            persistent_log: Some(PersistentLog {
                dir: log_path.path().to_path_buf(),
            }),
        }),
    )
    .unwrap();

    let qcli = QCli::default();

    let correct_fragment_builder = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        BlockDate::first().next_epoch(),
    );

    let faulty_fragment_builder = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        BlockDate::first(),
    );

    // Should be rejected without a log entry
    qcli.fragment_sender(&quibitous)
        .send(
            &faulty_fragment_builder
                .transaction(&sender, receiver.address(), 100.into())
                .unwrap()
                .encode(),
        )
        .assert_rejected_summary();

    // Should be accepted with a log entry
    qcli.fragment_sender(&quibitous)
        .send(
            &correct_fragment_builder
                .transaction(&sender, receiver.address(), 101.into())
                .unwrap()
                .encode(),
        )
        .assert_in_block();

    // Should be rejected without a log entry
    qcli.fragment_sender(&quibitous)
        .send(
            &faulty_fragment_builder
                .transaction(&sender, receiver.address(), 102.into())
                .unwrap()
                .encode(),
        )
        .assert_rejected_summary();

    assert_eq!(
        PersistentLogViewer::new(log_path.path().to_path_buf()).count(),
        1
    );
}
