//! The `version` subcommand

#![allow(clippy::never_loop)]

use super::KmsCommand;
use abscissa::{Command, Runnable};

/// The `version` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct VersionCommand {}

impl Runnable for VersionCommand {
    /// Print version message
    fn run(&self) {
        println!("{} {}", KmsCommand::name(), KmsCommand::version());
    }
}
