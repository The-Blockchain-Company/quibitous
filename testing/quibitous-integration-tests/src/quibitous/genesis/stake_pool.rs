use chain_crypto::{RistrettoGroup2HashDh, SumEd25519_12};
use chain_impl_mockchain::fee::LinearFee;
use quibitous_automation::testing::keys;
use quibitous_automation::{
    qcli::QCli,
    quibitous::{ConfigurationBuilder, QuibitousProcess, Starter},
};
use quibitous_lib::{
    crypto::hash::Hash,
    interfaces::{BlockDate, InitialUTxO, Ratio, TaxType, Value},
};
use jortestkit::process::Wait;
use silica::Wallet;

use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::str::FromStr;

#[test]
pub fn create_delegate_retire_stake_pool() {
    let temp_dir = TempDir::new().unwrap();

    let mut actor_account = silica::Wallet::default();

    let config = ConfigurationBuilder::new()
        .with_linear_fees(LinearFee::new(100, 100, 200))
        .with_funds(vec![InitialUTxO {
            value: 1_000_000.into(),
            address: actor_account.address(),
        }])
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config.clone())
        .start()
        .unwrap();

    let stake_pool_id = create_new_stake_pool(
        &mut actor_account,
        config.genesis_block_hash(),
        BlockDate::new(1, 0),
        &quibitous,
        &Default::default(),
    );
    delegate_stake(
        &mut actor_account,
        &stake_pool_id,
        config.genesis_block_hash(),
        BlockDate::new(1, 0),
        &quibitous,
        &Default::default(),
    );
    retire_stake_pool(
        &stake_pool_id,
        &mut actor_account,
        config.genesis_block_hash(),
        BlockDate::new(1, 0),
        &quibitous,
        &Default::default(),
    );
}

pub fn create_new_stake_pool(
    account: &mut Wallet,
    genesis_block_hash: &str,
    valid_until: BlockDate,
    quibitous: &QuibitousProcess,
    wait: &Wait,
) -> String {
    let temp_dir = TempDir::new().unwrap();
    let qcli: QCli = Default::default();

    let kes = keys::create_new_key_pair::<RistrettoGroup2HashDh>();
    let vrf = keys::create_new_key_pair::<SumEd25519_12>();

    let owner_stake_key = temp_dir.child("stake_key.private_key");
    owner_stake_key
        .write_str(&account.signing_key_to_string())
        .unwrap();

    let settings = qcli.rest().v0().settings(quibitous.rest_uri());
    let fees: LinearFee = settings.fees;
    let fee_value: Value = (fees.certificate + fees.coefficient + fees.constant).into();

    let stake_pool_certificate = qcli.certificate().new_stake_pool_registration(
        &vrf.identifier().to_bech32_str(),
        &kes.identifier().to_bech32_str(),
        0u32,
        1u32,
        &account.identifier().to_bech32_str(),
        Some(TaxType {
            fixed: 0.into(),
            ratio: Ratio::new_checked(1, 2).unwrap(),
            max_limit: None,
        }),
    );
    let stake_pool_certificate_file = temp_dir.child("stake_pool.cert");
    stake_pool_certificate_file
        .write_str(&stake_pool_certificate)
        .unwrap();
    let block0_hash = Hash::from_hex(genesis_block_hash).unwrap();
    let transaction = qcli
        .transaction_builder(block0_hash)
        .new_transaction()
        .add_account(&account.address().to_string(), &fee_value)
        .add_certificate(&stake_pool_certificate)
        .set_expiry_date(valid_until)
        .finalize_with_fee(&account.address().to_string(), &fees)
        .seal_with_witness_data(account.witness_data())
        .add_auth(owner_stake_key.path())
        .to_message();

    account.confirm_transaction();
    qcli.fragment_sender(quibitous)
        .send(&transaction)
        .assert_in_block_with_wait(wait);

    let stake_pool_id = qcli
        .certificate()
        .stake_pool_id(stake_pool_certificate_file.path());

    assert!(
        qcli.rest()
            .v0()
            .stake_pools(&quibitous.rest_uri())
            .contains(&stake_pool_id),
        "cannot find stake-pool certificate in blockchain"
    );

    stake_pool_id
}

pub fn delegate_stake(
    account: &mut Wallet,
    stake_pool_id: &str,
    genesis_block_hash: &str,
    valid_until: BlockDate,
    quibitous: &QuibitousProcess,
    wait: &Wait,
) {
    let temp_dir = TempDir::new().unwrap();
    let qcli: QCli = Default::default();

    let owner_stake_key = temp_dir.child("stake_key.private_key");
    owner_stake_key
        .write_str(&account.signing_key_to_string())
        .unwrap();

    let stake_pool_delegation = qcli
        .certificate()
        .new_stake_delegation(stake_pool_id, &account.identifier().to_bech32_str());

    let settings = qcli.rest().v0().settings(&quibitous.rest_uri());
    let fees: LinearFee = settings.fees;
    let fee_value: Value = (fees.certificate + fees.coefficient + fees.constant).into();
    let block0_hash = Hash::from_hex(genesis_block_hash).unwrap();

    let transaction = qcli
        .transaction_builder(block0_hash)
        .new_transaction()
        .add_account(&account.address().to_string(), &fee_value)
        .add_certificate(&stake_pool_delegation)
        .set_expiry_date(valid_until)
        .finalize_with_fee(&account.address().to_string(), &fees)
        .seal_with_witness_data(account.witness_data())
        .add_auth(owner_stake_key.path())
        .to_message();

    account.confirm_transaction();
    qcli.fragment_sender(quibitous)
        .send(&transaction)
        .assert_in_block_with_wait(wait);

    let account_state_after_delegation = qcli
        .rest()
        .v0()
        .account_stats(account.address().to_string(), quibitous.rest_uri());

    let stake_pool_id_hash = Hash::from_str(stake_pool_id).unwrap();
    assert!(
        account_state_after_delegation
            .delegation()
            .pools()
            .iter()
            .any(|(hash, _)| *hash == stake_pool_id_hash),
        "account should be delegated to pool"
    );
}

pub fn retire_stake_pool(
    stake_pool_id: &str,
    account: &mut Wallet,
    genesis_block_hash: &str,
    valid_until: BlockDate,
    quibitous: &QuibitousProcess,
    wait: &Wait,
) {
    let temp_dir = TempDir::new().unwrap();
    let qcli: QCli = Default::default();

    let owner_stake_key = temp_dir.child("stake_key.private_key");
    owner_stake_key
        .write_str(&account.signing_key_to_string())
        .unwrap();

    let retirement_cert = qcli.certificate().new_stake_pool_retirement(stake_pool_id);

    let settings = qcli.rest().v0().settings(quibitous.rest_uri());
    let fees: LinearFee = settings.fees;
    let fee_value: Value = (fees.certificate + fees.coefficient + fees.constant).into();
    let block0_hash = Hash::from_hex(genesis_block_hash).unwrap();

    let transaction = qcli
        .transaction_builder(block0_hash)
        .new_transaction()
        .add_account(&account.address().to_string(), &fee_value)
        .add_certificate(&retirement_cert)
        .set_expiry_date(valid_until)
        .finalize_with_fee(&account.address().to_string(), &fees)
        .seal_with_witness_data(account.witness_data())
        .add_auth(owner_stake_key.path())
        .to_message();

    account.confirm_transaction();
    qcli.fragment_sender(quibitous)
        .send(&transaction)
        .assert_in_block_with_wait(wait);

    let account_state_after_stake_pool_retire = qcli
        .rest()
        .v0()
        .account_stats(account.address().to_string(), quibitous.rest_uri());

    let stake_pool_id_hash = Hash::from_str(stake_pool_id).unwrap();

    assert!(
        account_state_after_stake_pool_retire
            .delegation()
            .pools()
            .iter()
            .any(|(hash, _)| *hash == stake_pool_id_hash),
        "account should be still delegated to retired pool"
    );

    assert!(
        !qcli
            .rest()
            .v0()
            .stake_pools(&quibitous.rest_uri())
            .contains(&stake_pool_id.to_owned()),
        "stake pool should not be listed among active stake pools"
    );
}
