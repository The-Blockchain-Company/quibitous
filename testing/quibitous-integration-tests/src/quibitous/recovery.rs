use quibitous_automation::{
    qcli::JCli,
    quibitous::{ConfigurationBuilder, QuibitousProcess, LogLevel, Starter},
};

use quibitous_automation::quibitous::LeadershipMode;
use quibitous_lib::interfaces::{AccountState, BlockDate, InitialUTxO, SettingsDto, UTxOInfo};
use silica::Wallet;

use assert_fs::prelude::*;
use assert_fs::TempDir;

#[derive(Clone, Debug, PartialEq)]
struct LedgerSnapshot {
    settings: SettingsDto,
    utxo_info: UTxOInfo,
    account_state: AccountState,
}

impl LedgerSnapshot {
    pub fn new(settings: SettingsDto, utxo_info: UTxOInfo, account_state: AccountState) -> Self {
        LedgerSnapshot {
            settings,
            utxo_info,
            account_state,
        }
    }
}

fn take_snapshot(
    account_receiver: &Wallet,
    quibitous: &QuibitousProcess,
    utxo_info: UTxOInfo,
) -> LedgerSnapshot {
    let qcli: JCli = Default::default();
    let rest_uri = quibitous.rest_uri();
    let settings = qcli.rest().v0().settings(&rest_uri);
    let account = qcli
        .rest()
        .v0()
        .account_stats(account_receiver.address().to_string(), &rest_uri);
    qcli.rest()
        .v0()
        .utxo()
        .assert_contains(&utxo_info, &rest_uri);

    LedgerSnapshot::new(settings, utxo_info, account)
}

pub fn do_simple_transaction(
    sender: &Wallet,
    account_receiver: &Wallet,
    utxo_sender: &UTxOInfo,
    utxo_receiver: &Wallet,
    quibitous: &QuibitousProcess,
) -> UTxOInfo {
    let qcli: JCli = Default::default();
    const TX_VALUE: u64 = 50;
    let mut tx = qcli.transaction_builder(quibitous.genesis_block_hash());
    let transaction_message = tx
        .new_transaction()
        .add_input_from_utxo(utxo_sender)
        .add_output(&account_receiver.address().to_string(), TX_VALUE.into())
        .add_output(&utxo_receiver.address().to_string(), TX_VALUE.into())
        .set_expiry_date(BlockDate::new(1, 0))
        .finalize()
        .seal_with_witness_data(sender.witness_data())
        .to_message();
    let tx_id = tx.fragment_id();

    qcli.fragment_sender(quibitous)
        .send(&transaction_message)
        .assert_in_block();

    UTxOInfo::new(tx_id, 1, utxo_receiver.address(), TX_VALUE.into())
}

#[test]
pub fn test_node_recovers_from_node_restart() {
    let temp_dir = TempDir::new().unwrap();
    let qcli: JCli = Default::default();

    let sender = silica::Wallet::new_utxo(&mut rand::rngs::OsRng);
    let account_receiver = silica::Wallet::default();
    let utxo_receiver = silica::Wallet::new_utxo(&mut rand::rngs::OsRng);

    let config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 100.into(),
        }])
        .with_storage(&temp_dir.child("storage"))
        .with_log_level(LogLevel::TRACE.to_string())
        .build(&temp_dir);

    let quibitous = Starter::new().config(config.clone()).start().unwrap();
    let utxo_sender = config.block0_utxo_for_address(&sender.address());

    let new_utxo = do_simple_transaction(
        &sender,
        &account_receiver,
        &utxo_sender,
        &utxo_receiver,
        &quibitous,
    );
    let snapshot_before = take_snapshot(&account_receiver, &quibitous, new_utxo.clone());
    qcli.rest().v0().shutdown(quibitous.rest_uri());

    std::thread::sleep(std::time::Duration::from_secs(5));

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config)
        .leadership_mode(LeadershipMode::Leader)
        .start()
        .unwrap();

    quibitous
        .rest()
        .raw()
        .send_until_ok(
            |raw| raw.account_state(&account_receiver.account_id()),
            Default::default(),
        )
        .expect("timeout occured when pooling address endpoint");

    let snapshot_after = take_snapshot(&account_receiver, &quibitous, new_utxo);

    assert_eq!(
        snapshot_before, snapshot_after,
        "Different snaphot after restart {:?} vs {:?}",
        snapshot_before, snapshot_after
    );
}

#[test]
pub fn test_node_recovers_kill_signal() {
    let temp_dir = TempDir::new().unwrap();

    let sender = silica::Wallet::new_utxo(&mut rand::rngs::OsRng);
    let account_receiver = silica::Wallet::default();
    let utxo_receiver = silica::Wallet::new_utxo(&mut rand::rngs::OsRng);

    let config = ConfigurationBuilder::new()
        .with_funds(vec![InitialUTxO {
            address: sender.address(),
            value: 100.into(),
        }])
        .with_storage(&temp_dir.child("storage"))
        .build(&temp_dir);

    let quibitous = Starter::new().config(config.clone()).start().unwrap();
    let utxo_sender = config.block0_utxo_for_address(&sender.address());

    let new_utxo = do_simple_transaction(
        &sender,
        &account_receiver,
        &utxo_sender,
        &utxo_receiver,
        &quibitous,
    );
    let snapshot_before = take_snapshot(&account_receiver, &quibitous, new_utxo.clone());
    // Wait before stopping so transactions are flushed to disk
    std::thread::sleep(std::time::Duration::from_secs(1));
    quibitous.stop();

    std::thread::sleep(std::time::Duration::from_secs(5));

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config)
        .leadership_mode(LeadershipMode::Leader)
        .start()
        .unwrap();

    quibitous
        .rest()
        .raw()
        .send_until_ok(
            |raw| raw.account_state(&account_receiver.account_id()),
            Default::default(),
        )
        .unwrap_or_else(|_| {
            panic!(
                "timeout occured when pooling address endpoint. \nNode logs: {}",
                quibitous.logger.get_log_content()
            )
        });

    let snapshot_after = take_snapshot(&account_receiver, &quibitous, new_utxo);

    assert_eq!(
        snapshot_before, snapshot_after,
        "Different snaphot after restart {:?} vs {:?}",
        snapshot_before, snapshot_after
    );
}
