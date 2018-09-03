use clear_on_drop::ClearOnDrop;
use signatory::{
    ed25519::{FromSeed, Seed, Signature, Signer as SignerTrait},
    providers::dalek::Ed25519Signer as DalekSigner,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    panic::RefUnwindSafe,
    path::Path,
};

use super::{PublicKey, Signer};
use config::ProviderConfig;
use error::Error;

#[cfg(feature = "dalek-provider")]
use super::signer::dalek;

#[cfg(feature = "yubihsm-provider")]
use super::signer::yubihsm;

pub struct Keyring {
    secret_connection_signer: DalekSigner,
    signing_keys: HashMap<PublicKey, Signer>,
}

impl Keyring {
    /// Create a keyring from the given provider configuration
    pub fn from_config(seccon_key_path: &Path, config: &ProviderConfig) -> Result<Self, Error> {
        let secret_connection_signer = create_seccon_signer(seccon_key_path)?;
        let mut signers = vec![];

        #[cfg(feature = "dalek-provider")]
        dalek::create_signers(&mut signers, config.dalek.as_ref())?;

        #[cfg(feature = "yubihsm-provider")]
        yubihsm::create_signers(&mut signers, &config.yubihsm)?;

        let mut signing_keys = HashMap::new();

        for mut signer in signers {
            let public_key = signer.public_key()?;
            info!(
                "Added {}:{} {}",
                signer.provider_name, signer.key_id, &public_key
            );
            signing_keys.insert(public_key, signer);
        }

        Ok(Self {
            secret_connection_signer,
            signing_keys,
        })
    }

    /// Get the signer which authenticates new `SecretConnection` sessions
    pub fn secret_connection_signer(&self) -> &DalekSigner {
        &self.secret_connection_signer
    }

    /// Sign a message using the secret key associated with the given public key
    /// (if it is in our keyring)
    pub fn sign(&self, public_key: &PublicKey, msg: &[u8]) -> Result<Signature, Error> {
        let signer = self
            .signing_keys
            .get(public_key)
            .ok_or_else(|| err!(InvalidKey, "not in keyring: {}", public_key))?;

        signer.sign(msg)
    }
}

// TODO: ensure keyring is actually unwind safe
// The `yubihsm-rs` crate uses interior mutability, for example, and
// therefore is generally not unwind safe, but should theoretically be
// panic-free barring any bugs in the implementation
impl RefUnwindSafe for Keyring {}

/// Load the key for the `SecretConnection`
fn create_seccon_signer(key_path: &Path) -> Result<DalekSigner, Error> {
    let seed = match File::open(key_path) {
        Ok(mut seed_file) => {
            let mut key_material = ClearOnDrop::new(vec![]);
            seed_file.read_to_end(key_material.as_mut())?;
            Seed::from_slice(&key_material)?
        }
        Err(_) => {
            let seed = Seed::generate();
            let mut seed_file = File::create(&key_path)?;
            seed_file.write_all(seed.as_secret_slice())?;
            seed
        }
    };

    let signer = DalekSigner::from_seed(seed);

    info!(
        "KMS node ID: {}",
        PublicKey::from(signer.public_key().unwrap())
    );

    Ok(signer)
}
