mod adversary;
mod tx_only;

use crate::mfive_lib::MfiveError;
use structopt::StructOpt;
pub use tx_only::TxOnly;
#[derive(StructOpt, Debug)]
pub enum Batch {
    /// Prints nodes related data, like stats,fragments etc.
    TxOnly(tx_only::TxOnly),
    Adversary(adversary::Adversary),
}

impl Batch {
    pub fn exec(&self) -> Result<(), MfiveError> {
        match self {
            Batch::TxOnly(tx_only_command) => tx_only_command.exec(),
            Batch::Adversary(adversary_command) => adversary_command.exec(),
        }
    }
}
