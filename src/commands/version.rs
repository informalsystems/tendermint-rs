//! The `version` subcommand

#![allow(clippy::never_loop)]

use super::KmsCommand;
use abscissa::{Command as CommandTrait, Runnable};

/// The `version` subcommand
#[derive(Debug, Default, Options)]
pub struct VersionCommand {}

impl Runnable for VersionCommand {
    /// Print version message
    fn run(&self) {
        KmsCommand::print_package_info();
    }
}
