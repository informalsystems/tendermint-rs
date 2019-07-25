//! `tmkms softsign import` command

use crate::{config::provider::softsign::KeyFormat, keyring::SecretKeyEncoding, prelude::*};
use abscissa_core::{Command, Runnable};
use signatory::{ed25519, Decode, Encode};
use std::{path::PathBuf, process};
use subtle_encoding::IDENTITY;
use tendermint::{config::PrivValidatorKey, PrivateKey};

/// `import` command: import a `priv_validator.json` formatted key and convert
/// it into the raw format used by the softsign backend (by default)
#[derive(Command, Debug, Default, Options)]
pub struct ImportCommand {
    #[options(
        short = "f",
        help = "key format to import: 'json' or 'raw' (default 'json')"
    )]
    format: Option<String>,

    #[options(free, help = "[INPUT] and [OUTPUT] paths for key generation")]
    paths: Vec<PathBuf>,
}

impl Runnable for ImportCommand {
    /// Import a `priv_validator.json`
    fn run(&self) {
        if self.paths.len() != 2 {
            status_err!("expected 2 arguments, got {}", self.paths.len());
            eprintln!("\nUsage: tmkms softsign import [priv_validator.json] [output.key]");
            process::exit(1);
        }

        let input_path = &self.paths[0];
        let output_path = &self.paths[1];

        let format = self
            .format
            .as_ref()
            .map(|f| {
                f.parse::<KeyFormat>().unwrap_or_else(|e| {
                    status_err!("{} (must be 'json' or 'raw')", e);
                    process::exit(1);
                })
            })
            .unwrap_or(KeyFormat::Json);

        let seed = match format {
            KeyFormat::Json => {
                let private_key = PrivValidatorKey::load_json_file(input_path)
                    .unwrap_or_else(|e| {
                        status_err!("couldn't load {}: {}", input_path.display(), e);
                        process::exit(1);
                    })
                    .priv_key;

                match private_key {
                    PrivateKey::Ed25519(pk) => pk.to_seed(),
                }
            }
            KeyFormat::Raw => {
                ed25519::Seed::decode_from_file(input_path, IDENTITY).unwrap_or_else(|e| {
                    status_err!("couldn't load {}: {}", input_path.display(), e);
                    process::exit(1);
                })
            }
            KeyFormat::Base64 => {
                status_err!("invalid format: baes64 (must be 'json' or 'raw')");
                process::exit(1);
            }
        };

        seed.encode_to_file(output_path, &SecretKeyEncoding::default())
            .unwrap_or_else(|e| {
                status_err!("couldn't write to {}: {}", output_path.display(), e);
                process::exit(1);
            });

        info!("Imported Ed25519 private key to {}", output_path.display());
    }
}
