use super::*;
use abscissa::Callable;
use chrono::{SecondsFormat, Utc};
use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process,
};
use subtle_encoding::base64;
use tendermint::PublicKey;

/// The `yubihsm keys generate` subcommand
#[derive(Debug, Default, Options)]
pub struct GenerateCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Label for generated key(s)
    #[options(short = "l", long = "label")]
    pub label: Option<String>,

    /// Bech32 prefix to use when displaying key
    #[options(short = "p", long = "prefix")]
    pub bech32_prefix: Option<String>,

    /// Type of key to generate (default 'ed25519')
    #[options(short = "t")]
    pub key_type: Option<String>,

    /// Mark this key as non-exportable
    #[options(no_short, long = "non-exportable")]
    pub non_exportable: bool,

    /// Create an encrypted backup of this key in the given file
    #[options(short = "b", long = "backup")]
    pub backup_file: Option<PathBuf>,

    /// Key ID of the wrap key to use when creating a backup
    #[options(short = "w", long = "wrapkey")]
    pub wrap_key_id: Option<yubihsm::object::Id>,

    /// Key ID to generate
    #[options(free)]
    key_ids: Vec<u16>,
}

impl Callable for GenerateCommand {
    /// Generate an Ed25519 signing key inside a YubiHSM2 device
    fn call(&self) {
        if self.key_ids.len() != 1 {
            status_err!(
                "expected exactly 1 key ID to generate, got {}",
                self.key_ids.len()
            );
            process::exit(1);
        }

        let key_id = self.key_ids[0];

        if let Some(key_type) = self.key_type.as_ref() {
            if key_type != DEFAULT_KEY_TYPE {
                status_err!(
                    "only supported key type is: ed25519 (given: \"{}\")",
                    key_type
                );
                process::exit(1);
            }
        }

        let hsm = crate::yubihsm::client();
        let mut capabilities = DEFAULT_CAPABILITIES;

        // If the key isn't explicitly marked as non-exportable, allow it to be exported
        if !self.non_exportable {
            capabilities |= yubihsm::Capability::EXPORTABLE_UNDER_WRAP;
        }

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        let label = yubihsm::object::Label::from(
            match self.label {
                Some(ref l) => l.to_owned(),
                None => match self.bech32_prefix {
                    Some(ref prefix) => format!("{}:{}", prefix, timestamp),
                    None => format!("ed25519:{}", timestamp),
                },
            }
            .as_ref(),
        );

        if let Err(e) = hsm.generate_asymmetric_key(
            key_id,
            label,
            DEFAULT_DOMAINS, // TODO(tarcieri): customize domains
            capabilities,
            yubihsm::asymmetric::Algorithm::Ed25519,
        ) {
            status_err!("couldn't generate key #{}: {}", key_id, e);
            process::exit(1);
        }

        let public_key = PublicKey::from_raw_ed25519(
            hsm.get_public_key(key_id)
                .unwrap_or_else(|e| {
                    status_err!("couldn't get public key for key #{}: {}", key_id, e);
                    process::exit(1);
                })
                .as_ref(),
        )
        .unwrap();

        let public_key_string = match self.bech32_prefix {
            Some(ref prefix) => public_key.to_bech32(prefix),
            None => public_key.to_hex(),
        };

        status_ok!("Generated", "key #{}: {}", key_id, public_key_string);

        if let Some(ref backup_file) = self.backup_file {
            create_encrypted_backup(
                &hsm,
                key_id,
                &backup_file,
                self.wrap_key_id.unwrap_or(DEFAULT_WRAP_KEY),
            );
        }
    }
}

// TODO: custom derive in abscissa
impl_command!(GenerateCommand);

/// Create an encrypted backup of this key under the given wrap key ID
// TODO(tarcieri): unify this with the similar code in export?
fn create_encrypted_backup(
    hsm: &yubihsm::Client,
    key_id: yubihsm::object::Id,
    backup_file_path: &Path,
    wrap_key_id: yubihsm::object::Id,
) {
    let wrapped_bytes = hsm
        .export_wrapped(wrap_key_id, yubihsm::object::Type::AsymmetricKey, key_id)
        .unwrap_or_else(|e| {
            status_err!(
                "couldn't export key {} under wrap key {}: {}",
                key_id,
                wrap_key_id,
                e
            );
            process::exit(1);
        });

    let mut backup_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(backup_file_path)
        .unwrap_or_else(|e| {
            status_err!(
                "couldn't create backup file: {} ({})",
                backup_file_path.display(),
                e
            );
            process::exit(1);
        });

    backup_file
        .write_all(&base64::encode(&wrapped_bytes.into_vec()))
        .unwrap_or_else(|e| {
            status_err!("error writing backup: {}", e);
            process::exit(1);
        });

    status_ok!(
        "Wrote",
        "backup of key {} (encrypted under wrap key {}) to {}",
        key_id,
        wrap_key_id,
        backup_file_path.display()
    );
}
