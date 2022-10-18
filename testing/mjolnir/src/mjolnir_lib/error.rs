use quibitous_automation::quibitous::{QuibitousError, RestError, StartupError};
use quibitous_automation::testing::block0::Block0Error;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MjolnirError {
    #[error("cannot query rest")]
    RestError(#[from] RestError),
    #[error("cannot bootstrap node")]
    StartupError(#[from] StartupError),
    #[error("quibitous error")]
    QuibitousError(#[from] QuibitousError),
    #[error("node client error")]
    InternalClientError,
    #[error("pace is too low ({0})")]
    PaceTooLow(u64),
    #[error("get block0 error")]
    Block0Error(#[from] Block0Error),
}
