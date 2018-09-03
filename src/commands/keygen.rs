use clear_on_drop::ClearOnDrop;
use rand::{OsRng, RngCore};
use std::{env, fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, process};

/// Unix file permissions required for private keys (i.e. owner-readable only)
pub const PRIVATE_KEY_PERMISSIONS: u32 = 0o600;

/// Options for the `keygen` command
#[derive(Debug, Default, Options)]
pub struct KeygenCommand {
    #[options(free)]
    output_paths: Vec<String>,
}

impl KeygenCommand {
    /// Generate an Ed25519 secret key for use with a software provider (i.e. ed25519-dalek)
    pub fn call(&self) {
        if self.output_paths.len() != 1 {
            eprintln!("Usage: {} keygen [PATH]", env::args().next().unwrap());
            process::exit(2);
        }

        let output_path = &self.output_paths[0];

        // Buffer which will receive the random seed value
        let mut seed = ClearOnDrop::new(vec![0u8; 32]);
        OsRng::new().unwrap().fill_bytes(seed.as_mut());

        let mut output_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(PRIVATE_KEY_PERMISSIONS)
            .open(output_path)
            .unwrap_or_else(|e| {
                status_err!("couldn't open {} for writing: {}", output_path, e);
                process::exit(1);
            });

        // TODO: some sort of serialization format for the private key? Raw is easy for now
        output_file.write_all(&*seed).unwrap_or_else(|e| {
            status_err!("couldn't write to {}: {}", output_path, e);
            process::exit(1);
        });

        info!("Wrote random Ed25519 private key to {}", output_path);
    }
}
