use crate::mfive_lib::MfiveError;
use structopt::StructOpt;
use thiserror::Error;

mod batch;
mod standard;

#[derive(StructOpt, Debug)]
pub enum FragmentLoadCommand {
    /// sends fragments using batch endpoint
    Batch(batch::Batch),
    /// sends fragments in single manner
    Standard(standard::Standard),
}

#[derive(Error, Debug)]
pub enum FragmentLoadCommandError {
    #[error("Client Error")]
    ClientError(#[from] MfiveError),
}

impl FragmentLoadCommand {
    pub fn exec(&self) -> Result<(), MfiveError> {
        match self {
            FragmentLoadCommand::Batch(batch) => batch.exec(),
            FragmentLoadCommand::Standard(standard) => standard.exec(),
        }
    }
}
