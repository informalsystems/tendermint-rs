//! `tmkms softsign` CLI (sub)commands

mod keygen;

use self::keygen::KeygenCommand;
use abscissa_core::{Command, Help, Runnable};

/// The `softsign` subcommand
#[derive(Command, Debug, Options, Runnable)]
pub enum SoftSignCommand {
    /// Show help for the `softsign` subcommand
    #[options(help = "show help for the 'yubihsm' subcommand")]
    Help(Help<Self>),

    /// Generate a software signing key
    #[options(help = "generate a software signing key")]
    Keygen(KeygenCommand),
}
