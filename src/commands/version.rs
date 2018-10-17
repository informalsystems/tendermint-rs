//! The `version` subcommand

use abscissa::{Callable, Command as CommandTrait};

use super::KmsCommand;

/// The `version` subcommand
#[derive(Debug, Default, Options)]
pub struct VersionCommand {}

impl Callable for VersionCommand {
    /// Print version message
    fn call(&self) {
        KmsCommand::print_package_info();
    }
}
