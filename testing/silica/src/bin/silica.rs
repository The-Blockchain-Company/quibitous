mod cli;

use cli::command::Command;
use structopt::StructOpt;
use silica::cli::CliController;

pub fn main() {
    let controller = CliController::new().unwrap();
    Command::from_args().exec(controller).unwrap();
}
