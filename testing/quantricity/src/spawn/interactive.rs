use crate::builder::NetworkBuilder;
use crate::builder::Topology;
use crate::controller::QuibitousInteractiveCommandExec;
use crate::controller::UserInteractionController;
use crate::{config::Config, error::Error};
use quibitestkit::prelude::UserInteraction;

pub fn spawn_network(config: Config, topology: Topology) -> Result<(), Error> {
    let controller = NetworkBuilder::default()
        .topology(topology)
        .blockchain_config(config.build_blockchain())
        .session_settings(config.session)
        .build()?;

    let user_integration = quibitous_user_interaction();

    let mut interactive_commands = QuibitousInteractiveCommandExec {
        controller: UserInteractionController::new(controller),
    };

    user_integration
        .interact(&mut interactive_commands)
        .map_err(Into::into)
}

fn quibitous_user_interaction() -> UserInteraction {
    UserInteraction::new(
        "quantricity".to_string(),
        "interactive mode".to_string(),
        "type command:".to_string(),
        "exit".to_string(),
        ">".to_string(),
        vec![
            "You can control each aspect of test:".to_string(),
            "- spawn nodes,".to_string(),
            "- send fragments,".to_string(),
            "- filter logs,".to_string(),
            "- show node stats and data.".to_string(),
        ],
    )
}
