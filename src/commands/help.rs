//! The `help` subcommand

use abscissa::{Command, Runnable};

use super::KmsCommand;

/// The `help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    /// Arguments to the `help` command
    #[options(free)]
    pub args: Vec<String>,
}

impl Runnable for HelpCommand {
    /// Print help message
    fn run(&self) {
        KmsCommand::print_usage(
            &self
                .args
                .as_slice()
                .iter()
                .map(|arg| arg.as_ref())
                .collect::<Vec<_>>(),
        );
    }
}
