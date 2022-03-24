mod all;
mod votes_only;

use crate::mfive_lib::MfiveError;
use structopt::StructOpt;
pub use votes_only::VotesOnly;
#[derive(StructOpt, Debug)]
pub enum Adversary {
    VotesOnly(votes_only::VotesOnly),
    All(all::AdversaryAll),
}

impl Adversary {
    pub fn exec(&self) -> Result<(), MfiveError> {
        match self {
            Adversary::VotesOnly(votes_only_command) => votes_only_command.exec(),
            Adversary::All(all_command) => all_command.exec(),
        }
    }
}
