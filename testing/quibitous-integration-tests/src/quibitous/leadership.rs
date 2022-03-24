use crate::startup;
use quibitous_automation::{
    qcli::JCli,
    quibitous::{ConfigurationBuilder, QuibitousProcess, StartupVerificationMode},
};
use quibitous_lib::interfaces::LeadershipLogStatus;
use std::time::Duration;

#[test]
fn verify_genesis_optimum_leadership_logs_parent_hash() {
    let faucet = silica::Wallet::default();
    let (quibitous, _) =
        startup::start_stake_pool(&[faucet], &[], &mut ConfigurationBuilder::new()).unwrap();

    verify_leadership_logs_parent_hash(quibitous);
}

#[test]
fn verify_bft_leadership_logs_parent_hash() {
    let quibitous = startup::start_bft(
        vec![&silica::Wallet::default()],
        &mut ConfigurationBuilder::new(),
    )
    .unwrap();

    verify_leadership_logs_parent_hash(quibitous);
}

fn verify_leadership_logs_parent_hash(quibitous: QuibitousProcess) {
    quibitous
        .wait_for_bootstrap(&StartupVerificationMode::Rest, Duration::from_secs(10))
        .unwrap();

    // Give the node some time to produce blocks
    std::thread::sleep(Duration::from_secs(5));

    let qcli = JCli::default();

    let leadership_logs = qcli.rest().v0().leadership_log(quibitous.rest_uri());

    // leadership logs are fetched in reverse order (newest first)
    for leadership in leadership_logs.iter().take(10).rev() {
        if let LeadershipLogStatus::Block { block, parent, .. } = leadership.status() {
            let actual_block =
                qcli.rest()
                    .v0()
                    .block()
                    .next(parent.to_string(), 1, quibitous.rest_uri());
            assert_eq!(actual_block, *block, "wrong parent block");
        }
    }
}
