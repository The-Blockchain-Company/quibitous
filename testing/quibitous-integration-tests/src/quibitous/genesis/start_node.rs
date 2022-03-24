use crate::startup;
use chain_crypto::Ed25519Extended;
use quibitous_automation::quibitous::ConfigurationBuilder;
use quibitous_automation::testing::keys;

#[test]
pub fn test_genesis_stake_pool_with_account_faucet_starts_successfully() {
    let faucet = silica::Wallet::default();
    let (_quibitous, _) =
        startup::start_stake_pool(&[faucet], &[], &mut ConfigurationBuilder::new()).unwrap();
}

#[test]
pub fn test_genesis_stake_pool_with_utxo_faucet_starts_successfully() {
    let stake_key = keys::create_new_key_pair::<Ed25519Extended>();
    let faucet = startup::create_new_delegation_address_for(&stake_key.identifier());
    let (_quibitous, _) =
        startup::start_stake_pool(&[faucet], &[], &mut ConfigurationBuilder::new()).unwrap();
}
