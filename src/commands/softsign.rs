//! `tmkms softsign` CLI (sub)commands

mod import;
mod keygen;

use self::{import::ImportCommand, keygen::KeygenCommand};
use abscissa_core::{Command, Help, Runnable};

/// The `softsign` subcommand
#[derive(Command, Debug, Options, Runnable)]
pub enum SoftsignCommand {
    /// Show help for the `softsign` subcommand
    #[options(help = "show help for the 'yubihsm' subcommand")]
    Help(Help<Self>),

    /// Generate a software signing key
    #[options(help = "generate a software signing key")]
    Keygen(KeygenCommand),

    /// Import an existing key into the softsign Base64 format
    #[options(help = "convert existing private key to base64 format")]
    Import(ImportCommand),
}
