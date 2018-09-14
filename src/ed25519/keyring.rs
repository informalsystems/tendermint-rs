use signatory::{
    self,
    ed25519::{Ed25519Signature, FromSeed, Seed},
    encoding::{Decode, Encode, Encoding},
};
use signatory_dalek::Ed25519Signer as DalekSigner;
use std::{collections::HashMap, panic::RefUnwindSafe, path::Path};

use super::{PublicKey, Signer};
use config::ProviderConfig;
use error::Error;

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

        let mut signers = Vec::<Signer>::new();
        dalek::create_signers(&mut signers, config.dalek.as_ref())?;

        #[cfg(feature = "yubihsm-provider")]
        yubihsm::create_signers(&mut signers, &config.yubihsm)?;

        let mut signing_keys = HashMap::new();

        for mut signer in signers {
            let public_key = signer.public_key;
            info!(
                "Added {}:{} {}",
                signer.provider_name, signer.key_id, signer.public_key
            );
            signing_keys.insert(signer.public_key, signer);
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
    pub fn sign(&self, public_key: &PublicKey, msg: &[u8]) -> Result<Ed25519Signature, Error> {
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

/// Signatory encoding for the secret key
const SECCON_KEY_ENCODING: Encoding = Encoding::Raw;

/// Load the key for the `SecretConnection`
fn create_seccon_signer(key_path: &Path) -> Result<DalekSigner, Error> {
    let seed = if key_path.exists() {
        Seed::decode_from_file(key_path, SECCON_KEY_ENCODING)?
    } else {
        let s = Seed::generate();
        s.encode_to_file(key_path, SECCON_KEY_ENCODING)?;
        s
    };

    let signer = DalekSigner::from_seed(seed);

    info!(
        "KMS node ID: {}",
        PublicKey::from(signatory::public_key(&signer)?)
    );

    Ok(signer)
}
