use crate::qcli_lib::{
    certificate::{weighted_pool_ids::WeightedPoolIds, write_cert, Error},
    utils::key_parser::parse_pub_key,
};
use chain_crypto::{Ed25519, PublicKey};
use chain_impl_mockchain::{
    certificate::{Certificate, StakeDelegation as Delegation},
    transaction::UnspecifiedAccountIdentifier,
};
use quibitous_lib::interfaces::Certificate as CertificateType;
use std::{convert::TryInto, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct StakeDelegation {
    /// the public key used in the stake key registration
    #[structopt(name = "STAKE_KEY", parse(try_from_str = parse_pub_key))]
    stake_id: PublicKey<Ed25519>,

    #[structopt(flatten)]
    pool_ids: WeightedPoolIds,

    /// write the output to the given file or print it to the standard output if not defined
    #[structopt(short = "o", long = "output")]
    output: Option<PathBuf>,
}

impl StakeDelegation {
    pub fn exec(self) -> Result<(), Error> {
        let content = Delegation {
            account_id: UnspecifiedAccountIdentifier::from_single_account(self.stake_id.into()),
            delegation: (&self.pool_ids).try_into()?,
        };
        let cert = Certificate::StakeDelegation(content);
        write_cert(self.output.as_deref(), CertificateType(cert))
    }
}
