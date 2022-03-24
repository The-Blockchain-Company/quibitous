use crate::startup;
use chain_addr::Discrimination;
use chain_core::property::BlockDate;
use chain_impl_mockchain::certificate::VoteTallyPayload;
use chain_impl_mockchain::tokens::minting_policy::MintingPolicy;
use chain_impl_mockchain::{certificate::VoteAction, fee::LinearFee, vote::Choice};
use quibitous_automation::quibitous::ConfigurationBuilder;
use quibitous_automation::testing::time;
use quibitous_automation::testing::VotePlanBuilder;
use quibitous_lib::interfaces::{AccountVotes, InitialToken};
use std::time::Duration;
use silica::FragmentSenderSetup;

#[test]
pub fn list_cast_votes_for_active_vote_plan() {
    let mut alice = silica::Wallet::default();
    let bob = silica::Wallet::default();
    let wait_time = Duration::from_secs(2);
    let discrimination = Discrimination::Test;

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(3)
        .action_type(VoteAction::OffChain)
        .vote_start(BlockDate::from_epoch_slot_id(1, 0))
        .tally_start(BlockDate::from_epoch_slot_id(20, 0))
        .tally_end(BlockDate::from_epoch_slot_id(30, 0))
        .public()
        .build();

    let quibitous = startup::start_bft(
        vec![&alice, &bob],
        ConfigurationBuilder::new()
            .with_discrimination(discrimination)
            .with_slots_per_epoch(20)
            .with_slot_duration(3)
            .with_linear_fees(LinearFee::new(0, 0, 0))
            .with_token(InitialToken {
                token_id: vote_plan.voting_token().clone().into(),
                policy: MintingPolicy::new().into(),
                to: vec![alice.to_initial_token(1_000)],
            }),
    )
    .unwrap();

    assert!(quibitous
        .rest()
        .account_votes_with_plan_id(vote_plan.to_id().into(), alice.address())
        .is_err());
    assert_eq!(
        Some(vec![]),
        quibitous.rest().account_votes(alice.address()).unwrap()
    );

    let proposals_ids = vec![0u8, 1u8, 2u8];

    silica::FragmentChainSender::from_with_setup(
        quibitous.block0_configuration(),
        quibitous.to_remote(),
        FragmentSenderSetup::no_verify(),
    )
    .send_vote_plan(&mut alice, &vote_plan)
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .then_wait_for_epoch(1)
    .cast_vote(&mut alice, &vote_plan, proposals_ids[0], &Choice::new(1))
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .cast_vote(&mut alice, &vote_plan, proposals_ids[1], &Choice::new(1))
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .cast_vote(&mut alice, &vote_plan, proposals_ids[2], &Choice::new(1))
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap();

    assert_eq!(
        Some(proposals_ids.clone()),
        quibitous
            .rest()
            .account_votes_with_plan_id(vote_plan.to_id().into(), alice.address())
            .unwrap()
    );
    assert_eq!(
        Some(vec![AccountVotes {
            vote_plan_id: vote_plan.to_id().into(),
            votes: proposals_ids
        }]),
        quibitous.rest().account_votes(alice.address()).unwrap()
    );
    assert_eq!(
        Some(vec![]),
        quibitous
            .rest()
            .account_votes_with_plan_id(vote_plan.to_id().into(), bob.address())
            .unwrap()
    );
    assert_eq!(
        Some(vec![AccountVotes {
            vote_plan_id: vote_plan.to_id().into(),
            votes: vec![]
        }]),
        quibitous.rest().account_votes(bob.address()).unwrap()
    );
}

#[test]
pub fn list_cast_votes_for_already_finished_vote_plan() {
    let mut alice = silica::Wallet::default();
    let wait_time = Duration::from_secs(2);
    let discrimination = Discrimination::Test;

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(3)
        .action_type(VoteAction::OffChain)
        .vote_start(BlockDate::from_epoch_slot_id(1, 0))
        .tally_start(BlockDate::from_epoch_slot_id(2, 0))
        .tally_end(BlockDate::from_epoch_slot_id(2, 1))
        .public()
        .build();

    let quibitous = startup::start_bft(
        vec![&alice],
        ConfigurationBuilder::new()
            .with_discrimination(discrimination)
            .with_slots_per_epoch(20)
            .with_slot_duration(3)
            .with_linear_fees(LinearFee::new(0, 0, 0))
            .with_token(InitialToken {
                token_id: vote_plan.voting_token().clone().into(),
                policy: MintingPolicy::new().into(),
                to: vec![alice.to_initial_token(1_000_000)],
            }),
    )
    .unwrap();

    let proposals_ids = vec![0u8, 1u8, 2u8];

    silica::FragmentChainSender::from_with_setup(
        quibitous.block0_configuration(),
        quibitous.to_remote(),
        FragmentSenderSetup::no_verify(),
    )
    .send_vote_plan(&mut alice, &vote_plan)
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .then_wait_for_epoch(1)
    .cast_vote(&mut alice, &vote_plan, 0, &Choice::new(1))
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .cast_vote(&mut alice, &vote_plan, 1, &Choice::new(1))
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .cast_vote(&mut alice, &vote_plan, 2, &Choice::new(1))
    .unwrap()
    .and_verify_is_in_block(wait_time)
    .unwrap()
    .then_wait_for_epoch(2)
    .tally_vote(&mut alice, &vote_plan, VoteTallyPayload::Public)
    .unwrap()
    .then_wait_for_epoch(3);

    assert_eq!(
        Some(proposals_ids.clone()),
        quibitous
            .rest()
            .account_votes_with_plan_id(vote_plan.to_id().into(), alice.address())
            .unwrap()
    );
    assert_eq!(
        Some(vec![AccountVotes {
            vote_plan_id: vote_plan.to_id().into(),
            votes: proposals_ids
        }]),
        quibitous.rest().account_votes(alice.address()).unwrap()
    );
}

#[test]
pub fn list_casted_votes_for_non_voted() {
    let alice = silica::Wallet::default();
    let discrimination = Discrimination::Test;

    let quibitous = startup::start_bft(
        vec![&alice],
        ConfigurationBuilder::new()
            .with_discrimination(discrimination)
            .with_slots_per_epoch(20)
            .with_slot_duration(3)
            .with_linear_fees(LinearFee::new(0, 0, 0)),
    )
    .unwrap();

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(3)
        .action_type(VoteAction::OffChain)
        .vote_start(BlockDate::from_epoch_slot_id(1, 0))
        .tally_start(BlockDate::from_epoch_slot_id(20, 0))
        .tally_end(BlockDate::from_epoch_slot_id(30, 0))
        .public()
        .build();

    time::wait_for_epoch(2, quibitous.rest());

    assert!(quibitous
        .rest()
        .account_votes_with_plan_id(vote_plan.to_id().into(), alice.address())
        .is_err());
    assert_eq!(
        Some(vec![]),
        quibitous.rest().account_votes(alice.address()).unwrap()
    );
}
