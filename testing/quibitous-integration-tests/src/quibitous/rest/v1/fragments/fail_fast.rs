use crate::startup;
use chain_impl_mockchain::{block::BlockDate, fragment::Fragment};
use quibitous_automation::quibitous::{
    assert_bad_request, ConfigurationBuilder, QuibitousProcess, MemPoolCheck,
};
use gate::FaultyTransactionBuilder;
use rstest::*;
use std::time::Duration;
use silica::FragmentSender;
use silica::FragmentVerifier;

#[fixture]
fn world() -> (
    QuibitousProcess,
    Fragment,
    Fragment,
    Fragment,
    Fragment,
    Fragment,
) {
    let alice = silica::Wallet::default();
    let bob = silica::Wallet::default();
    let clarice = silica::Wallet::default();
    let david = silica::Wallet::default();

    let (quibitous, _stake_pools) = startup::start_stake_pool(
        &[alice.clone()],
        &[bob.clone(), clarice.clone()],
        &mut ConfigurationBuilder::new(),
    )
    .unwrap();

    let fragment_builder = silica::FragmentBuilder::new(
        &quibitous.genesis_block_hash(),
        &quibitous.fees(),
        BlockDate::first().next_epoch(),
    );

    let alice_fragment = fragment_builder
        .transaction(&alice, bob.address(), 100.into())
        .unwrap();

    let bob_fragment = fragment_builder
        .transaction(&bob, alice.address(), 100.into())
        .unwrap();
    let clarice_fragment = fragment_builder
        .transaction(&clarice, alice.address(), 100.into())
        .unwrap();

    let late_invalid_fragment = fragment_builder
        .transaction(&david, alice.address(), 100.into())
        .unwrap();

    let faulty_tx_builder = FaultyTransactionBuilder::new(
        quibitous.genesis_block_hash(),
        quibitous.fees(),
        BlockDate::first().next_epoch().into(),
    );
    let early_invalid_fragment = faulty_tx_builder.unbalanced(&alice, &bob);

    (
        quibitous,
        alice_fragment,
        bob_fragment,
        clarice_fragment,
        early_invalid_fragment,
        late_invalid_fragment,
    )
}

#[rstest]
pub fn fail_fast_on_all_valid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, valid_fragment_3, _, _) = world;
    let transaction_sender = FragmentSender::from(quibitous.block0_configuration());
    let tx_ids: Vec<MemPoolCheck> = transaction_sender
        .send_batch_fragments(
            vec![valid_fragment_1, valid_fragment_2, valid_fragment_3],
            true,
            &quibitous,
        )
        .unwrap()
        .fragment_ids()
        .into_iter()
        .map(MemPoolCheck::from)
        .collect();

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_all_valid(&tx_ids);
}

#[rstest]
pub fn fail_fast_off_all_valid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, valid_fragment_3, _, _) = world;
    let transaction_sender = FragmentSender::from(quibitous.block0_configuration());
    let tx_ids: Vec<MemPoolCheck> = transaction_sender
        .send_batch_fragments(
            vec![valid_fragment_1, valid_fragment_2, valid_fragment_3],
            false,
            &quibitous,
        )
        .unwrap()
        .fragment_ids()
        .into_iter()
        .map(MemPoolCheck::from)
        .collect();

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_all_valid(&tx_ids);
}

#[rstest]
pub fn fail_fast_on_first_invalid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, early_invalid_fragment, _) = world;
    assert_bad_request(quibitous.rest().send_fragment_batch(
        vec![early_invalid_fragment, valid_fragment_1, valid_fragment_2],
        true,
    ));

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_no_fragments();
}

#[rstest]
pub fn fail_fast_on_first_late_invalid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, _, late_invalid_fragment) = world;

    let fragments = vec![late_invalid_fragment, valid_fragment_1, valid_fragment_2];

    FragmentSender::from(quibitous.block0_configuration())
        .send_batch_fragments(fragments.clone(), true, &quibitous)
        .unwrap();

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_invalid(&fragments[0].hash().into())
        .assert_valid(&fragments[1].hash().into())
        .assert_valid(&fragments[2].hash().into());
}

#[rstest]
pub fn fail_fast_off_first_invalid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, early_invalid_fragment, _) = world;
    let tx_ids = assert_bad_request(quibitous.rest().send_fragment_batch(
        vec![valid_fragment_1, valid_fragment_2, early_invalid_fragment],
        true,
    ));

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_not_exist(&tx_ids[2])
        .assert_valid(&tx_ids[0])
        .assert_valid(&tx_ids[1]);
}

#[rstest]
pub fn fail_fast_off_invalid_in_middle(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, early_invalid_fragment, _) = world;
    let tx_ids = assert_bad_request(quibitous.rest().send_fragment_batch(
        vec![valid_fragment_1, early_invalid_fragment, valid_fragment_2],
        false,
    ));

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_valid(&tx_ids[0])
        .assert_valid(&tx_ids[2])
        .assert_not_exist(&tx_ids[1]);
}

#[rstest]
pub fn fail_fast_on_invalid_in_middle(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, early_invalid_fragment, _) = world;
    let tx_ids = assert_bad_request(quibitous.rest().send_fragment_batch(
        vec![valid_fragment_1, early_invalid_fragment, valid_fragment_2],
        true,
    ));

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_valid(&tx_ids[0])
        .assert_not_exist(&tx_ids[1])
        .assert_not_exist(&tx_ids[2]);
}
#[rstest]
pub fn fail_fast_on_last_invalid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, early_invalid_fragment, _) = world;
    let tx_ids = assert_bad_request(quibitous.rest().send_fragment_batch(
        vec![valid_fragment_1, valid_fragment_2, early_invalid_fragment],
        true,
    ));

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_valid(&tx_ids[0])
        .assert_valid(&tx_ids[1])
        .assert_not_exist(&tx_ids[2]);
}

#[rstest]
pub fn fail_fast_off_last_invalid(
    world: (
        QuibitousProcess,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
        Fragment,
    ),
) {
    let (quibitous, valid_fragment_1, valid_fragment_2, _, early_invalid_fragment, _) = world;
    let tx_ids = assert_bad_request(quibitous.rest().send_fragment_batch(
        vec![valid_fragment_1, valid_fragment_2, early_invalid_fragment],
        false,
    ));

    FragmentVerifier::wait_for_all_fragments(Duration::from_secs(5), &quibitous).unwrap();

    quibitous
        .correct_state_verifier()
        .fragment_logs()
        .assert_valid(&tx_ids[0])
        .assert_valid(&tx_ids[1])
        .assert_not_exist(&tx_ids[2]);
}
