use super::Error;
use crate::{
    qcli::{QCli, QCliCommand},
    quibitous::QuibitousProcess,
};
use chain_impl_mockchain::fragment::FragmentId;
use quibitous_lib::{
    crypto::hash::Hash,
    interfaces::{FragmentLog, FragmentStatus, FragmentsProcessingSummary},
};
use quibitestkit::prelude::ProcessOutput;
use quibitestkit::process::{run_process_until_response_matches, Wait};
use std::process::Command;

pub struct FragmentCheck<'a> {
    qcli: QCli,
    id: FragmentId,
    quibitous: &'a QuibitousProcess,
    summary: FragmentsProcessingSummary,
}

impl<'a> FragmentCheck<'a> {
    pub fn new(
        qcli: QCli,
        quibitous: &'a QuibitousProcess,
        id: FragmentId,
        summary: FragmentsProcessingSummary,
    ) -> Self {
        Self {
            qcli,
            id,
            quibitous,
            summary,
        }
    }

    pub fn fragment_id(&self) -> FragmentId {
        self.id
    }

    pub fn assert_in_block(&self) -> FragmentId {
        let wait: Wait = Default::default();
        self.assert_in_block_with_wait(&wait)
    }

    pub fn assert_in_block_with_wait(&self, wait: &Wait) -> FragmentId {
        self.wait_until_processed(wait).unwrap();
        self.assert_log_shows_in_block()
    }

    pub fn assert_rejected(&self, expected_reason: &str) {
        let wait: Wait = Default::default();
        self.wait_until_processed(&wait).unwrap();
        self.assert_log_shows_rejected(expected_reason);
    }

    pub fn wait_until_processed(&self, wait: &Wait) -> Result<FragmentId, Error> {
        run_process_until_response_matches(
            QCliCommand::new(Command::new(self.qcli.path()))
                .rest()
                .v0()
                .message()
                .logs(self.quibitous.rest_uri())
                .build(),
            |output| {
                let content = output.as_lossy_string();
                let fragments: Vec<FragmentLog> =
                    serde_yaml::from_str(&content).expect("Cannot parse fragment logs");
                match fragments
                    .iter()
                    .find(|x| *x.fragment_id() == Hash::from_hash(self.id))
                {
                    Some(x) => {
                        println!("Transaction found in mempool. {:?}", x);
                        !x.is_pending()
                    }
                    None => {
                        println!("Transaction with hash {} not found in mempool", self.id);
                        false
                    }
                }
            },
            wait.sleep_duration().as_secs(),
            wait.attempts(),
            &format!(
                "Waiting for transaction: '{}' to be inBlock or rejected",
                self.id
            ),
            &format!(
                "transaction: '{}' is pending for too long, Logs: {:?}",
                self.id,
                self.quibitous.logger.get_log_content()
            ),
        )
        .map(|()| self.id)
        .map_err(|_| Error::TransactionNotInBlock {
            message_log: format!(
                "{:?}",
                self.qcli
                    .rest()
                    .v0()
                    .message()
                    .logs(&self.quibitous.rest_uri())
            ),
            transaction_id: Hash::from_hash(self.id),
            log_content: self.quibitous.logger.get_log_content(),
        })
    }

    fn assert_log_shows_in_block(&self) -> FragmentId {
        let fragments = self
            .qcli
            .rest()
            .v0()
            .message()
            .logs(self.quibitous.rest_uri());
        match fragments
            .iter()
            .find(|x| *x.fragment_id() == Hash::from_hash(self.id))
        {
            Some(x) => assert!(
                x.is_in_a_block(),
                "Fragment should be in block, actual: {:?}. Logs: {:?}",
                &x,
                self.quibitous.logger.get_log_content()
            ),
            None => panic!(
                "cannot find any fragment in rest message log, output: {:?}. Node log: {:?}",
                &fragments,
                self.quibitous.logger.get_log_content()
            ),
        }
        self.id
    }

    pub fn assert_log_shows_rejected(&self, expected_msg: &str) {
        let fragments = self
            .qcli
            .rest()
            .v0()
            .message()
            .logs(self.quibitous.rest_uri());
        match fragments
            .iter()
            .find(|x| *x.fragment_id() == Hash::from_hash(self.id))
        {
            Some(x) => {
                assert!(
                    x.is_rejected(),
                    "Fragment should be rejected, actual: {:?}. Logs: {:?}",
                    &x,
                    self.quibitous.logger.get_log_content()
                );
                match x.status() {
                    FragmentStatus::Rejected { reason } => assert!(reason.contains(&expected_msg)),
                    _ => panic!("Non expected state for for rejected log"),
                }
            }
            None => panic!(
                "cannot find any fragment in rest message log, output: {:?}. Logs: {:?}",
                &fragments,
                self.quibitous.logger.get_log_content()
            ),
        }
    }

    pub fn assert_rejected_summary(&self) {
        assert!(
            self.summary.rejected.iter().any(|i| i.id == self.id),
            "expected transaction '{}' to be rejected. Log: {:?}",
            self.id,
            self.summary
        );
    }
}
