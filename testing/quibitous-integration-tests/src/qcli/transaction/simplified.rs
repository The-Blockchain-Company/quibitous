use quibitous_automation::quibitous::ConfigurationBuilder;
use quibitous_automation::{qcli::QCli, quibitous::Starter};
use quibitous_lib::crypto::hash::Hash;
use quibitous_lib::interfaces::{BlockDate, InitialUTxO};

use assert_fs::TempDir;
use chain_impl_mockchain::fee::LinearFee;
use std::io::Write;

#[test]
pub fn test_make_test_transaction() {
    let temp_dir = TempDir::new().unwrap();

    let qcli: QCli = Default::default();
    let sender = silica::Wallet::default();

    let sk_file_path = temp_dir.join("sender.sk");

    {
        let mut sk_file = std::fs::File::create(&sk_file_path).unwrap();
        sk_file
            .write_all(sender.signing_key_to_string().as_bytes())
            .unwrap();
    }

    let staging_file = temp_dir.join("staging.txt");

    let config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 100.into(),
        }])
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config.clone())
        .start()
        .unwrap();

    let block0_hash = Hash::from_hex(config.genesis_block_hash()).unwrap();

    qcli.transaction().make_transaction(
        quibitous.rest_uri(),
        sender.address(),
        None,
        100.into(),
        block0_hash.to_string(),
        BlockDate::new(1, 0),
        sk_file_path,
        staging_file,
        false,
    );
}

#[test]
pub fn test_make_transaction_to_receiver_account() {
    let temp_dir = TempDir::new().unwrap();

    let qcli: QCli = Default::default();
    let sender = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let sk_file_path = temp_dir.join("sender.sk");

    {
        let mut sk_file = std::fs::File::create(&sk_file_path).unwrap();
        sk_file
            .write_all(sender.signing_key_to_string().as_bytes())
            .unwrap();
    }

    let staging_file = temp_dir.join("staging.txt");

    let config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 100.into(),
        }])
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config.clone())
        .start()
        .unwrap();

    let block0_hash = Hash::from_hex(config.genesis_block_hash()).unwrap();

    qcli.transaction().make_transaction(
        quibitous.rest_uri(),
        sender.address(),
        Some(receiver.address()),
        100.into(),
        block0_hash.to_string(),
        BlockDate::new(1, 0),
        sk_file_path,
        staging_file,
        false,
    );
}

#[test]
pub fn test_make_transaction_to_receiver_account_with_fees() {
    let temp_dir = TempDir::new().unwrap();

    let qcli: QCli = Default::default();
    let sender = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let sk_file_path = temp_dir.join("sender.sk");

    {
        let mut sk_file = std::fs::File::create(&sk_file_path).unwrap();
        sk_file
            .write_all(sender.signing_key_to_string().as_bytes())
            .unwrap();
    }

    let staging_file = temp_dir.join("staging.txt");

    let config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 111.into(),
        }])
        .with_linear_fees(LinearFee::new(10, 0, 0))
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config.clone())
        .start()
        .unwrap();

    let block0_hash = Hash::from_hex(config.genesis_block_hash()).unwrap();

    qcli.transaction().make_transaction(
        quibitous.rest_uri(),
        sender.address(),
        Some(receiver.address()),
        100.into(),
        block0_hash.to_string(),
        BlockDate::new(1, 0),
        sk_file_path,
        staging_file,
        false,
    );
}

#[test]
pub fn test_make_transaction_to_receiver_account_with_fees_and_post() {
    let temp_dir = TempDir::new().unwrap();

    let qcli: QCli = Default::default();
    let sender = silica::Wallet::default();
    let receiver = silica::Wallet::default();

    let sk_file_path = temp_dir.join("sender.sk");

    {
        let mut sk_file = std::fs::File::create(&sk_file_path).unwrap();
        sk_file
            .write_all(sender.signing_key_to_string().as_bytes())
            .unwrap();
    }

    let staging_file = temp_dir.join("staging.txt");

    let config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 111.into(),
        }])
        .with_linear_fees(LinearFee::new(10, 0, 0))
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config.clone())
        .start()
        .unwrap();

    let block0_hash = Hash::from_hex(config.genesis_block_hash()).unwrap();

    qcli.transaction().make_transaction(
        quibitous.rest_uri(),
        sender.address(),
        Some(receiver.address()),
        100.into(),
        block0_hash.to_string(),
        BlockDate::new(1, 0),
        sk_file_path,
        staging_file,
        true,
    );
}
