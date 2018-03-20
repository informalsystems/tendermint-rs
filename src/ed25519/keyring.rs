use std::collections::HashMap;

use error::Error;
use super::{PublicKey, Signer};

pub struct Keyring {
    keys: HashMap<PublicKey, Signer>,
}

impl Keyring {
    pub fn from_signers(signers: Vec<Signer>) -> Result<Self, Error> {
        let mut keys = HashMap::new();

        for mut signer in signers {
            keys.insert(signer.public_key()?, signer);
        }

        Ok(Self { keys })
    }
}
