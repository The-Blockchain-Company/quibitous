use crate::startup;
use chain_impl_mockchain::rewards::TaxType;
use quibitous_automation::testing::time;
use quibitous_automation::{qcli::JCli, quibitous::ConfigurationBuilder};
use silica::TransactionHash;

use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
pub fn update_pool_fees_is_not_allowed() {
    let temp_dir = TempDir::new().unwrap();
    let qcli: JCli = Default::default();

    let stake_pool_owner = silica::Wallet::default();

    let (quibitous, stake_pools) = startup::start_stake_pool(
        &[stake_pool_owner.clone()],
        &[],
        ConfigurationBuilder::new().with_storage(&temp_dir.child("storage")),
    )
    .unwrap();

    let stake_pool = stake_pools.get(0).unwrap();

    let mut new_stake_pool = stake_pool.clone();
    let mut stake_pool_info = new_stake_pool.info_mut();
    stake_pool_info.rewards = TaxType::zero();

    // 6. send pool update certificate
    time::wait_for_epoch(2, quibitous.rest());

    let transaction = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        chain_impl_mockchain::block::BlockDate {
            epoch: 3,
            slot_id: 0,
        },
    )
    .stake_pool_update(vec![&stake_pool_owner], stake_pool, &new_stake_pool)
    .encode();

    qcli.fragment_sender(&quibitous)
        .send(&transaction)
        .assert_rejected("Pool update doesnt currently allow fees update");
}
