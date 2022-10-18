mod load;
mod noise;
#[cfg(feature = "soak")]
mod soak;

use crate::non_functional::voting::config::PrivateVotingLoadTestConfig;
use assert_fs::TempDir;
use chain_core::property::BlockDate as _;
use chain_impl_mockchain::block::BlockDate;
use chain_impl_mockchain::testing::data::CommitteeMembersManager;
use chain_impl_mockchain::{
    certificate::{VoteAction, VoteTallyPayload},
    ledger::governance::TreasuryGovernanceAction,
    testing::decrypt_tally,
    value::Value,
};
use quibitous_automation::quibitous::{ConfigurationBuilder, Starter};
use quibitous_automation::testing::time::{wait_for_date, wait_for_epoch};
use quibitous_automation::testing::{benchmark_consumption, VotePlanBuilder};
use quibitous_lib::interfaces::BlockDate as BlockDateLib;
use jortestkit::load::Configuration;
use jortestkit::measurement::Status;
use gate::AdversaryFragmentSender;
use gate::AdversaryFragmentSenderSetup;
use mfive::generators::{AdversaryFragmentGenerator, FragmentStatusProvider, VoteCastsGenerator};
use rand::rngs::OsRng;
use silica::BlockDateGenerator;
use silica::{vote_plan_cert, FragmentSender, FragmentSenderSetup, Wallet};

const CRS_SEED: &[u8] = "Testing seed".as_bytes();

pub fn private_vote_load_scenario(quick_config: PrivateVotingLoadTestConfig) {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let mut rng = OsRng;
    let members = CommitteeMembersManager::new(
        &mut rng,
        CRS_SEED,
        quick_config.tally_threshold(),
        quick_config.members_count(),
    );

    let committee_keys = members
        .members()
        .iter()
        .map(|committee_member| committee_member.public_key())
        .collect::<Vec<_>>();

    let voters: Vec<Wallet> = std::iter::from_fn(|| Some(Wallet::default()))
        .take(quick_config.wallets_count())
        .collect();

    let mut committee = Wallet::default();

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(quick_config.proposals_count())
        .action_type(VoteAction::Treasury {
            action: TreasuryGovernanceAction::TransferToRewards {
                value: Value(quick_config.rewards_increase()),
            },
        })
        .vote_start(BlockDate::from_epoch_slot_id(
            quick_config.voting_timing()[0],
            0,
        ))
        .tally_start(BlockDate::from_epoch_slot_id(
            quick_config.voting_timing()[1],
            0,
        ))
        .tally_end(BlockDate::from_epoch_slot_id(
            quick_config.voting_timing()[2],
            0,
        ))
        .private()
        .member_public_keys(committee_keys)
        .build();

    let vote_plan_cert = vote_plan_cert(
        &committee,
        chain_impl_mockchain::block::BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        &vote_plan,
    )
    .into();

    let config = ConfigurationBuilder::new()
        .with_fund(committee.to_initial_fund(quick_config.initial_fund_per_wallet()))
        .with_funds_split_if_needed(
            voters
                .iter()
                .map(|x| x.to_initial_fund(quick_config.initial_fund_per_wallet()))
                .collect(),
        )
        .with_committees(&[committee.to_committee_id()])
        .with_slots_per_epoch(quick_config.slots_in_epoch())
        .with_certs(vec![vote_plan_cert])
        .with_slot_duration(quick_config.slot_duration())
        .with_block_content_max_size(quick_config.block_content_max_size().into())
        .with_treasury(1_000.into())
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config)
        .start()
        .unwrap();

    let settings = quibitous.rest().settings().unwrap();
    let block_date_generator = BlockDateGenerator::rolling(
        &settings,
        BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        false,
    );

    let transaction_sender = FragmentSender::new(
        quibitous.genesis_block_hash(),
        quibitous.fees(),
        block_date_generator,
        FragmentSenderSetup::no_verify(),
    );

    let benchmark_consumption_monitor = benchmark_consumption(&quick_config.measurement_name())
        .target(quick_config.target_resources_usage())
        .for_process("Node", quibitous.pid() as usize)
        .start_async(std::time::Duration::from_secs(30));

    let votes_generator = VoteCastsGenerator::new(
        voters,
        vote_plan.clone(),
        quibitous.to_remote(),
        transaction_sender.clone(),
    );

    let stats = jortestkit::load::start_async(
        votes_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        quick_config.configuration(),
        &quick_config.measurement_name(),
    );

    stats.print_summary(&quick_config.measurement_name());
    assert_eq!(
        stats
            .measure(
                &quick_config.measurement_name(),
                quick_config.tx_target_success_rate()
            )
            .status(),
        Status::Green
    );

    wait_for_epoch(quick_config.voting_timing()[1], quibitous.rest());

    wait_for_date(
        BlockDateLib::new(
            quick_config.voting_timing()[1],
            quick_config.slots_in_epoch() / 2,
        ),
        quibitous.rest(),
    );

    let active_vote_plans = quibitous.rest().vote_plan_statuses().unwrap();
    let vote_plan_status = active_vote_plans
        .iter()
        .find(|c_vote_plan| c_vote_plan.id == vote_plan.to_id().into())
        .unwrap();

    let shares = decrypt_tally(&vote_plan_status.clone().into(), &members).unwrap();

    transaction_sender
        .send_vote_tally(
            &mut committee,
            &vote_plan,
            &quibitous,
            VoteTallyPayload::Private { inner: shares },
        )
        .unwrap();

    wait_for_epoch(quick_config.voting_timing()[2], quibitous.rest());

    benchmark_consumption_monitor.stop();

    quibitous.assert_no_errors_in_log();
}

pub fn adversary_private_vote_load_scenario(
    quick_config: PrivateVotingLoadTestConfig,
    adversary_noise_config: Configuration,
) {
    let temp_dir = TempDir::new().unwrap().into_persistent();
    let mut rng = OsRng;
    let members = CommitteeMembersManager::new(
        &mut rng,
        CRS_SEED,
        quick_config.tally_threshold(),
        quick_config.members_count(),
    );

    let committee_keys = members
        .members()
        .iter()
        .map(|committee_member| committee_member.public_key())
        .collect::<Vec<_>>();

    let mut noise_wallet_from = Wallet::default();

    let voters: Vec<Wallet> = std::iter::from_fn(|| Some(Wallet::default()))
        .take(quick_config.wallets_count())
        .collect();

    let mut committee = Wallet::default();

    let vote_plan = VotePlanBuilder::new()
        .proposals_count(quick_config.proposals_count())
        .action_type(VoteAction::Treasury {
            action: TreasuryGovernanceAction::TransferToRewards {
                value: Value(quick_config.rewards_increase()),
            },
        })
        .vote_start(BlockDate::from_epoch_slot_id(
            quick_config.voting_timing()[0],
            0,
        ))
        .tally_start(BlockDate::from_epoch_slot_id(
            quick_config.voting_timing()[1],
            0,
        ))
        .tally_end(BlockDate::from_epoch_slot_id(
            quick_config.voting_timing()[2],
            0,
        ))
        .private()
        .member_public_keys(committee_keys)
        .build();

    let vote_plan_cert = vote_plan_cert(
        &committee,
        chain_impl_mockchain::block::BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        &vote_plan,
    )
    .into();

    let config = ConfigurationBuilder::new()
        .with_funds(vec![
            noise_wallet_from.to_initial_fund(1_000_000_000),
            committee.to_initial_fund(quick_config.initial_fund_per_wallet()),
        ])
        .with_funds_split_if_needed(
            voters
                .iter()
                .map(|x| x.to_initial_fund(quick_config.initial_fund_per_wallet()))
                .collect(),
        )
        .with_committees(&[committee.to_committee_id()])
        .with_slots_per_epoch(quick_config.slots_in_epoch())
        .with_certs(vec![vote_plan_cert])
        .with_slot_duration(quick_config.slot_duration())
        .with_block_content_max_size(quick_config.block_content_max_size().into())
        .with_treasury(1_000.into())
        .build(&temp_dir);

    let quibitous = Starter::new()
        .temp_dir(temp_dir)
        .config(config)
        .start()
        .unwrap();

    let settings = quibitous.rest().settings().unwrap();
    let block_date_generator = BlockDateGenerator::rolling(
        &settings,
        BlockDate {
            epoch: 1,
            slot_id: 0,
        },
        false,
    );

    let transaction_sender = FragmentSender::new(
        quibitous.genesis_block_hash(),
        quibitous.fees(),
        block_date_generator,
        FragmentSenderSetup::no_verify(),
    );

    let adversary_transaction_sender = AdversaryFragmentSender::new(
        quibitous.genesis_block_hash(),
        quibitous.fees(),
        chain_impl_mockchain::block::BlockDate {
            epoch: 1,
            slot_id: 0,
        }
        .into(),
        AdversaryFragmentSenderSetup::no_verify(),
    );

    let benchmark_consumption_monitor = benchmark_consumption(&quick_config.measurement_name())
        .target(quick_config.target_resources_usage())
        .for_process("Node", quibitous.pid() as usize)
        .start_async(std::time::Duration::from_secs(30));

    let mut adversary_votes_generator = AdversaryFragmentGenerator::new(
        quibitous.to_remote(),
        transaction_sender.clone(),
        adversary_transaction_sender,
    );

    adversary_votes_generator.fill_from_faucet(&mut noise_wallet_from);

    let _noise = jortestkit::load::start_background_async(
        adversary_votes_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        adversary_noise_config,
        "noise fragments",
    );

    let votes_generator = VoteCastsGenerator::new(
        voters,
        vote_plan.clone(),
        quibitous.to_remote(),
        transaction_sender.clone(),
    );

    let stats = jortestkit::load::start_async(
        votes_generator,
        FragmentStatusProvider::new(quibitous.to_remote()),
        quick_config.configuration(),
        &quick_config.measurement_name(),
    );

    stats.print_summary(&quick_config.measurement_name());
    assert_eq!(
        stats
            .measure(
                &quick_config.measurement_name(),
                quick_config.tx_target_success_rate()
            )
            .status(),
        Status::Green
    );

    wait_for_epoch(quick_config.voting_timing()[1], quibitous.rest());

    wait_for_date(
        BlockDateLib::new(
            quick_config.voting_timing()[1],
            quick_config.slots_in_epoch() / 2,
        ),
        quibitous.rest(),
    );

    let active_vote_plans = quibitous.rest().vote_plan_statuses().unwrap();
    let vote_plan_status = active_vote_plans
        .iter()
        .find(|c_vote_plan| c_vote_plan.id == vote_plan.to_id().into())
        .unwrap();

    let shares = decrypt_tally(&vote_plan_status.clone().into(), &members).unwrap();

    transaction_sender
        .send_vote_tally(
            &mut committee,
            &vote_plan,
            &quibitous,
            VoteTallyPayload::Private { inner: shares },
        )
        .unwrap();

    wait_for_epoch(quick_config.voting_timing()[2], quibitous.rest());

    benchmark_consumption_monitor.stop();

    quibitous.assert_no_errors_in_log();
}
