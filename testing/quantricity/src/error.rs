use crate::controller::Error as ControllerError;
use quibitous_automation::quibitous::{ExplorerError, RestError};
use quibitous_automation::testing::ConsumptionBenchmarkError;
use quibitestkit::prelude::InteractiveCommandError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    MonitorNode(#[from] crate::controller::NodeError),
    #[error(transparent)]
    InteractiveCommand(#[from] InteractiveCommandError),
    #[error(transparent)]
    Controller(#[from] ControllerError),
    #[error(transparent)]
    Verification(#[from] quibitous_automation::testing::VerificationError),
    #[error(transparent)]
    FragmentVerifier(#[from] silica::FragmentVerifierError),
    #[error(transparent)]
    ConsumptionBenchmark(#[from] ConsumptionBenchmarkError),
    #[error(transparent)]
    Explorer(#[from] ExplorerError),
    #[error(transparent)]
    FragmentSender(#[from] silica::FragmentSenderError),
    #[error("Rest error: {0}")]
    Rest(#[from] RestError),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yaml::Error),
    #[error("Circular dependency in network topology")]
    CircularTrust,
    #[error("INTERNAL ERROR: {0}")]
    Internal(String),
}
