use crate::controller::InteractiveCommandError;
use quibitous_automation::quibitous::ExplorerError;
use quibitous_automation::quibitous::LegacyConfigConverterError;
use quibitous_automation::quibitous::StartupError;
use thiserror::Error;
use silica::FragmentSenderError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Node(#[from] super::monitor::NodeError),

    #[error(transparent)]
    Wallet(#[from] silica::WalletError),

    #[error(transparent)]
    FsFixture(#[from] assert_fs::fixture::FixtureError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Explorer(#[from] ExplorerError),

    #[error(transparent)]
    BlockFormatError(#[from] chain_core::property::ReadError),

    #[error(transparent)]
    BlockWriteError(#[from] chain_core::property::WriteError),

    #[error("No node with alias {0}")]
    NodeNotFound(String),

    #[error("Wallet '{0}' was not found. Used before or never initialize")]
    WalletNotFound(String),

    #[error("StakePool '{0}' was not found. Used before or never initialize")]
    StakePoolNotFound(String),

    #[error("VotePlan '{0}' was not found. Used before or never initialize")]
    VotePlanNotFound(String),

    #[error(transparent)]
    Startup(#[from] StartupError),

    #[error("cannot spawn the node")]
    CannotSpawnNode(#[source] std::io::Error),

    #[error(transparent)]
    LegacyConfigConverter(#[from] LegacyConfigConverterError),

    #[error(transparent)]
    InteractiveCommand(#[from] InteractiveCommandError),

    #[error(transparent)]
    FragmentSender(#[from] FragmentSenderError),

    #[error(transparent)]
    Serialization(#[from] serde_yaml::Error),
}
