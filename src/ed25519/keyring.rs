use std::collections::HashMap;

use super::{PublicKey, Signer};

pub struct Keyring<'a> {
    keys: HashMap<PublicKey, Signer<'a>>,
}

impl<'a> Keyring<'a> {
    pub fn from_signers(signers: Vec<Signer<'a>>) -> Self {
        let mut keys = HashMap::new();

        for mut signer in signers {
            keys.insert(*signer.public_key(), signer);
        }

        Self { keys }
    }
}
