use super::YubihsmCommand;
use abscissa::{Command, Runnable};

/// The `yubihsm help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    /// Arguments to the `yubihsm help` command
    #[options(free)]
    pub args: Vec<String>,
}

impl Runnable for HelpCommand {
    /// Print help for the `yubihsm` subcommand
    fn run(&self) {
        YubihsmCommand::print_usage(
            &self
                .args
                .as_slice()
                .iter()
                .map(|arg| arg.as_ref())
                .collect::<Vec<_>>(),
        );
    }
}
