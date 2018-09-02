//! The `version` subcommand

#![allow(unknown_lints, never_loop)]

use abscissa::Command as CommandTrait;

use super::KMSCommand;

/// The `version` subcommand
#[derive(Debug, Default, Options)]
pub struct VersionCommand {}

impl VersionCommand {
    /// Print version message
    pub fn call(&self) {
        KMSCommand::print_package_info();
    }
}
