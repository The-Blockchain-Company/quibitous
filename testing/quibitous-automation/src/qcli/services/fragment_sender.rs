use super::{FragmentCheck, FragmentsCheck};
use crate::{qcli::JCli, quibitous::QuibitousProcess};

pub struct FragmentSender<'a> {
    qcli: JCli,
    quibitous: &'a QuibitousProcess,
}

impl<'a> FragmentSender<'a> {
    pub fn new(qcli: JCli, quibitous: &'a QuibitousProcess) -> Self {
        Self { qcli, quibitous }
    }

    pub fn send(self, transaction: &'a str) -> FragmentCheck {
        let summary = self
            .qcli
            .rest()
            .v0()
            .message()
            .post(transaction, self.quibitous.rest_uri());

        let id = if summary.accepted.len() == 1 {
            summary.accepted[0]
        } else if summary.rejected.len() == 1 {
            summary.rejected[0].id
        } else {
            panic!("Single transaction was sent but multiple or no processing results found");
        };

        FragmentCheck::new(self.qcli, self.quibitous, id, summary)
    }

    pub fn send_many(self, transactions: &'a [String]) -> FragmentsCheck {
        for tx in transactions {
            self.qcli
                .rest()
                .v0()
                .message()
                .post(tx, self.quibitous.rest_uri());
        }
        FragmentsCheck::new(self.qcli, self.quibitous)
    }
}
