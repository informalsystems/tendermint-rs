use crate::keyring::SecretKeyEncoding;
use abscissa::Runnable;
use signatory::{ed25519, Encode};
use std::{env, process};

/// Options for the `keygen` command
#[derive(Debug, Default, Options)]
pub struct KeygenCommand {
    #[options(free)]
    output_paths: Vec<String>,
}

impl Runnable for KeygenCommand {
    /// Generate an Ed25519 secret key for use with a software provider (i.e. ed25519-dalek)
    fn run(&self) {
        if self.output_paths.len() != 1 {
            eprintln!("Usage: {} keygen [PATH]", env::args().next().unwrap());
            process::exit(2);
        }

        let output_path = &self.output_paths[0];

        let seed = ed25519::Seed::generate();
        seed.encode_to_file(output_path, &SecretKeyEncoding::default())
            .unwrap_or_else(|e| {
                status_err!("couldn't write to {}: {}", output_path, e);
                process::exit(1);
            });

        info!("Wrote random Ed25519 private key to {}", output_path);
    }
}
