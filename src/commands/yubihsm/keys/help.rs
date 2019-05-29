use super::KeysCommand;
use abscissa::{Command, Runnable};

/// The `yubihsm keys help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    #[options(free)]
    pub args: Vec<String>,
}

impl Runnable for HelpCommand {
    /// Print help for the `yubihsm` subcommand
    fn run(&self) {
        KeysCommand::print_usage(
            &self
                .args
                .as_slice()
                .iter()
                .map(|arg| arg.as_ref())
                .collect::<Vec<_>>(),
        );
    }
}
