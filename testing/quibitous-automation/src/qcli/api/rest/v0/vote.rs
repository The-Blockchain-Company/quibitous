use crate::qcli::command::rest::v0::VoteCommand;
use assert_cmd::assert::OutputAssertExt;
use quibitous_lib::interfaces::{CommitteeIdDef, VotePlanStatus};
use quibitestkit::prelude::ProcessOutput;

pub struct Vote {
    vote_command: VoteCommand,
}

impl Vote {
    pub fn new(vote_command: VoteCommand) -> Self {
        Self { vote_command }
    }

    pub fn active_voting_committees<S: Into<String>>(self, host: S) -> Vec<CommitteeIdDef> {
        let content = self
            .vote_command
            .active_committees(host)
            .build()
            .assert()
            .success()
            .get_output()
            .as_lossy_string();
        serde_yaml::from_str(&content).expect("QCLI returned malformed CommitteeIdDef")
    }

    pub fn active_vote_plans<S: Into<String>>(self, host: S) -> Vec<VotePlanStatus> {
        let content = self
            .vote_command
            .active_vote_plans(host)
            .build()
            .assert()
            .success()
            .get_output()
            .as_lossy_string();
        serde_yaml::from_str(&content).expect("QCLI returned malformed VotePlan")
    }
}
