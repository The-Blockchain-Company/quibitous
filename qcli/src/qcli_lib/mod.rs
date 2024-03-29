pub mod address;
pub mod auto_completion;
pub mod block;
pub mod certificate;
pub mod debug;
pub mod key;
pub mod rest;
pub mod transaction;
pub mod vote;

pub mod utils;

use std::error::Error;
use structopt::StructOpt;

/// Quibitous CLI toolkit
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct QCli {
    /// display full version details (software version, source version, targets and compiler used)
    #[structopt(long = "full-version")]
    full_version: bool,

    /// display the sources version, allowing to check the source's hash used to compile this executable.
    /// this option is useful for scripting retrieving the logs of the version of this application.
    #[structopt(long = "source-version")]
    source_version: bool,

    #[structopt(subcommand)]
    command: Option<QCliCommand>,
}

#[allow(clippy::large_enum_variant)]
/// Quibitous CLI toolkit
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum QCliCommand {
    /// Key Generation
    Key(key::Key),
    /// Address tooling and helper
    Address(address::Address),
    /// Block tooling and helper
    Genesis(block::Genesis),
    /// Send request to node REST API
    Rest(rest::Rest),
    /// Build and view offline transaction
    Transaction(transaction::Transaction),
    /// Debug tools for developers
    Debug(debug::Debug),
    /// Certificate generation tool
    Certificate(certificate::Certificate),
    /// Auto completion
    AutoCompletion(auto_completion::AutoCompletion),
    /// Utilities that perform specialized tasks
    Utils(utils::Utils),
    /// Vote related operations
    Votes(vote::Vote),
}

impl QCli {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        use std::io::Write as _;
        if self.full_version {
            Ok(writeln!(std::io::stdout(), "{}", env!("FULL_VERSION"))?)
        } else if self.source_version {
            Ok(writeln!(std::io::stdout(), "{}", env!("SOURCE_VERSION"))?)
        } else if let Some(cmd) = self.command {
            cmd.exec()
        } else {
            writeln!(std::io::stderr(), "No command, try `--help'")?;
            std::process::exit(1);
        }
    }
}

impl QCliCommand {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        use self::QCliCommand::*;
        match self {
            Key(key) => key.exec()?,
            Address(address) => address.exec()?,
            Genesis(genesis) => genesis.exec()?,
            Rest(rest) => rest.exec()?,
            Transaction(transaction) => transaction.exec()?,
            Debug(debug) => debug.exec()?,
            Certificate(certificate) => certificate.exec()?,
            AutoCompletion(auto_completion) => auto_completion.exec::<Self>()?,
            Utils(utils) => utils.exec()?,
            Votes(vote) => vote.exec()?,
        };
        Ok(())
    }
}
