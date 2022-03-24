mod api;
mod command;
mod data;
mod services;

pub use command::JCliCommand;
pub use data::{Witness, WitnessData, WitnessType};
pub use services::FragmentCheck;

pub use services::Error;

use super::quibitous::QuibitousProcess;
use crate::testing::configuration;
use api::{Address, Certificate, Genesis, Key, Rest, Transaction, Votes};
use quibitous_lib::crypto::hash::Hash;
pub use services::{CertificateBuilder, FragmentSender, FragmentsCheck, TransactionBuilder};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug)]
pub struct JCli {
    exe: PathBuf,
}

impl Default for JCli {
    fn default() -> Self {
        Self::new(configuration::get_qcli_app())
    }
}

impl JCli {
    pub fn new(exe: PathBuf) -> Self {
        Self { exe }
    }

    pub fn path(&self) -> &Path {
        self.exe.as_path()
    }

    pub fn genesis(&self) -> Genesis {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Genesis::new(qcli_command.genesis())
    }

    pub fn key(&self) -> Key {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Key::new(qcli_command.key())
    }

    pub fn address(&self) -> Address {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Address::new(qcli_command.address())
    }

    pub fn rest(&self) -> Rest {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Rest::new(qcli_command.rest())
    }

    pub fn transaction(&self) -> Transaction {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Transaction::new(qcli_command.transaction())
    }

    pub fn certificate(&self) -> Certificate {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Certificate::new(qcli_command.certificate())
    }

    pub fn votes(&self) -> Votes {
        let command = Command::new(self.exe.clone());
        let qcli_command = JCliCommand::new(command);
        Votes::new(qcli_command.votes())
    }

    pub fn fragment_sender<'a>(&self, quibitous: &'a QuibitousProcess) -> FragmentSender<'a> {
        FragmentSender::new(self.clone(), quibitous)
    }

    pub fn transaction_builder(&self, genesis_hash: Hash) -> TransactionBuilder {
        TransactionBuilder::new(self.clone(), genesis_hash)
    }

    pub fn certificate_builder(&self) -> CertificateBuilder {
        CertificateBuilder::new(self.clone())
    }

    pub fn fragments_checker<'a>(&self, quibitous: &'a QuibitousProcess) -> FragmentsCheck<'a> {
        FragmentsCheck::new(self.clone(), quibitous)
    }
}
