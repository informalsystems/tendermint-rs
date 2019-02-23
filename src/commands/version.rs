//! The `version` subcommand

#![allow(clippy::never_loop)]

use super::KmsCommand;
use abscissa::{Callable, Command as CommandTrait};

/// The `version` subcommand
#[derive(Debug, Default, Options)]
pub struct VersionCommand {}

impl Callable for VersionCommand {
    /// Print version message
    fn call(&self) {
        KmsCommand::print_package_info();
    }
}
