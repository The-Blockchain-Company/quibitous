use assert_fs::fixture::PathChild;
use assert_fs::TempDir;
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::quibitous::{
    download_last_n_releases, get_quibitous_bin, ConfigurationBuilder, Starter, Version,
};
use quibitous_automation::testing::Release;
use quibitous_lib::interfaces::InitialUTxO;
use silica::{FragmentSender, TransactionHash};

fn test_connectivity_between_git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56"_and_legacy_app(release: Release, temp_dir: &TempDir) {
    println!("Testing version: {}", release.version());

    let sender = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let leader_config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 100.into(),
        }])
        .build(temp_dir);

    let leader_quibitous = Starter::new()
        .config(leader_config.clone())
        .start()
        .unwrap();

    let trusted_node_config = ConfigurationBuilder::new()
        .with_trusted_peers(vec![leader_quibitous.to_trusted_peer()])
        .with_block_hash(leader_config.genesis_block_hash())
        .build(temp_dir);

    let trusted_quibitous = Starter::new()
        .config(trusted_node_config)
        .legacy(release.version())
        .quibitous_app(get_quibitous_bin(&release, temp_dir))
        .passive()
        .start()
        .unwrap();

    let new_transaction = silica::FragmentBuilder::new(
        &leader_quibitous.genesis_block_hash(),
        &leader_quibitous.fees(),
        BlockDate::first().next_epoch(),
    )
    .transaction(&sender, receiver.address(), 1.into())
    .unwrap()
    .encode();

    let message = format!(
        "Unable to connect newest git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56" with node from {} version",
        release.version()
    );
    assert!(
        super::check_transaction_was_processed(new_transaction, &receiver, 1, &leader_quibitous)
            .is_ok(),
        "{}",
        message
    );

    trusted_quibitous.assert_no_errors_in_log_with_message("newest git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56" has errors in log");
    leader_quibitous.assert_no_errors_in_log_with_message(&format!(
        "Legacy nodes from {} version, has errrors in logs",
        release.version()
    ));
}

#[test]
// Re-enable when rate of breaking changes subsides and we can maintain
// backward compatible releases again.
#[ignore]
pub fn test_compability() {
    let temp_dir = TempDir::new().unwrap();
    for release in download_last_n_releases(5) {
        test_connectivity_between_git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56"_and_legacy_app(release, &temp_dir);
    }
}

#[test]
pub fn test_upgrade_downgrade() {
    let temp_dir = TempDir::new().unwrap();
    for release in download_last_n_releases(1) {
        test_upgrade_and_downgrade_from_legacy_to_git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56"(release.version(), &temp_dir);
    }
}

fn test_upgrade_and_downgrade_from_legacy_to_git+https://github.com/the-blockchain-company/chain-libs.git?branch=main#45b943be97f8bad0c90318a72cf23fc20d923d56"(version: Version, temp_dir: &TempDir) {
    println!("Testing version: {}", version);

    let mut sender = silica::Wallet::default();
    let mut receiver = silica::Wallet::default();

    let config = ConfigurationBuilder::new()
        .with_funds(vec![
            sender.to_initial_fund(1_000_000),
            receiver.to_initial_fund(1_000_000),
        ])
        .with_storage(&temp_dir.child("storage"))
        .build(temp_dir);

    // build some storage data on legacy node
    let legacy_quibitous = Starter::new()
        .config(config.clone())
        .legacy(version.clone())
        .start()
        .unwrap();

    let fragment_sender = FragmentSender::new(
        legacy_quibitous.genesis_block_hash(),
        legacy_quibitous.fees(),
        BlockDate::first().next_epoch().into(),
        Default::default(),
    );

    fragment_sender
        .send_transactions_round_trip(
            10,
            &mut sender,
            &mut receiver,
            &legacy_quibitous,
            100.into(),
        )
        .expect("fragment send error for legacy version");

    legacy_quibitous.assert_no_errors_in_log();

    legacy_quibitous.shutdown();

    // upgrade node to newest

    let quibitous = Starter::new().config(config.clone()).start().unwrap();

    fragment_sender
        .send_transactions_round_trip(10, &mut sender, &mut receiver, &quibitous, 100.into())
        .expect("fragment send error for legacy version");

    quibitous.assert_no_errors_in_log();
    quibitous.shutdown();

    // rollback node to legacy again

    let legacy_quibitous = Starter::new()
        .config(config)
        .legacy(version)
        .start()
        .unwrap();

    let fragment_sender = FragmentSender::new(
        legacy_quibitous.genesis_block_hash(),
        legacy_quibitous.fees(),
        BlockDate::first().next_epoch().into(),
        Default::default(),
    );

    fragment_sender
        .send_transactions_round_trip(
            1,
            &mut sender,
            &mut receiver,
            &legacy_quibitous,
            100.into(),
        )
        .expect("fragment send error for legacy version");

    legacy_quibitous.assert_no_errors_in_log();
    legacy_quibitous.shutdown();
}
