//! `tmkms softsign keygen` subcommand

use crate::{keyring::SecretKeyEncoding, prelude::*};
use abscissa_core::{Command, Runnable};
use signatory::{ed25519, Encode};
use std::{path::PathBuf, process};

/// `keygen` command
#[derive(Command, Debug, Default, Options)]
pub struct KeygenCommand {
    #[options(free, help = "path where generated key should be created")]
    output_paths: Vec<PathBuf>,
}

impl Runnable for KeygenCommand {
    /// Generate an Ed25519 secret key for use with a software provider (i.e. ed25519-dalek)
    fn run(&self) {
        if self.output_paths.len() != 1 {
            eprintln!("Usage: tmkms softsign keygen [PATH]");
            process::exit(1);
        }

        let output_path = &self.output_paths[0];

        let seed = ed25519::Seed::generate();
        seed.encode_to_file(output_path, &SecretKeyEncoding::default())
            .unwrap_or_else(|e| {
                status_err!("couldn't write to {}: {}", output_path.display(), e);
                process::exit(1);
            });

        info!(
            "Wrote random Ed25519 private key to {}",
            output_path.display()
        );
    }
}
