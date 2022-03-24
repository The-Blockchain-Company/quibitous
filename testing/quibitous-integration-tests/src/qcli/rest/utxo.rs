use crate::startup;
use assert_fs::TempDir;
use quibitous_automation::{
    qcli::JCli,
    quibitous::{ConfigurationBuilder, Starter},
};
use quibitous_lib::interfaces::InitialUTxO;

#[test]
pub fn test_correct_utxos_are_read_from_node() {
    let qcli: JCli = Default::default();
    let sender_utxo_address = startup::create_new_utxo_address();
    let receiver_utxo_address = startup::create_new_utxo_address();

    let funds = vec![
        InitialUTxO {
            address: receiver_utxo_address.address(),
            value: 100.into(),
        },
        InitialUTxO {
            address: sender_utxo_address.address(),
            value: 100.into(),
        },
    ];

    let temp_dir = TempDir::new().unwrap();

    let config = ConfigurationBuilder::new()
        .with_funds(funds)
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config.clone())
        .start()
        .unwrap();
    let rest_addr = quibitous.rest_uri();

    let sender_block0_utxo = config.block0_utxo_for_address(&sender_utxo_address.address());
    qcli.rest()
        .v0()
        .utxo()
        .assert_contains(&sender_block0_utxo, &rest_addr);

    let receiver_block0_utxo = config.block0_utxo_for_address(&receiver_utxo_address.address());
    qcli.rest()
        .v0()
        .utxo()
        .assert_contains(&receiver_block0_utxo, &rest_addr);
}
