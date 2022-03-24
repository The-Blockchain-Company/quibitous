use crate::startup;
use chain_core::property::Fragment;
use chain_impl_mockchain::block::BlockDate;
use quibitous_automation::quibitous::ConfigurationBuilder;
use quibitous_automation::quibitous::QuibitousProcess;
use quibitous_lib::interfaces::FragmentsProcessingSummary;
use rstest::*;
use silica::Wallet;

#[fixture]
fn world() -> (QuibitousProcess, Wallet, Wallet, Wallet) {
    let alice = silica::Wallet::default();
    let bob = silica::Wallet::default();
    let clarice = silica::Wallet::default();

    let (quibitous, _stake_pools) = startup::start_stake_pool(
        &[alice.clone()],
        &[bob.clone()],
        &mut ConfigurationBuilder::new(),
    )
    .unwrap();

    (quibitous, alice, bob, clarice)
}

#[rstest]
pub fn fragment_already_in_log(world: (QuibitousProcess, Wallet, Wallet, Wallet)) {
    let (quibitous, alice, bob, _) = world;

    let alice_fragment = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        BlockDate::first().next_epoch(),
    )
    .transaction(&alice, bob.address(), 100.into())
    .unwrap();

    let response = quibitous
        .rest()
        .raw()
        .send_fragment_batch(vec![alice_fragment.clone(), alice_fragment.clone()], false)
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let summary: FragmentsProcessingSummary =
        serde_json::from_str(&response.text().unwrap()).unwrap();
    assert_eq!(summary.accepted, vec![alice_fragment.id()]);
    assert_eq!(summary.rejected, vec![]);
}
