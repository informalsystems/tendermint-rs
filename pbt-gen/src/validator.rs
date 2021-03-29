//! Provides [proptest](https://github.com/AltSysrq/proptest) generators for
//! tendermint validator.

pub use ed25519_dalek::SecretKey;
use proptest::prelude::*;
use std::convert::TryFrom;
use tendermint::{
    account::Id,
    public_key::Ed25519,
    validator::{Info, Set},
    vote::Power,
    PublicKey,
};

prop_compose! {
    pub fn arb_id()
    (a in prop::array::uniform20(0u8..))
    -> Id {
        Id::try_from(a.to_vec()).unwrap()
    }
}

prop_compose! {
    pub fn arb_pub_key()
    (a in prop::array::uniform32(1u8..))
    -> PublicKey {
        let secret = SecretKey::from_bytes(&a).unwrap();
        let public = Ed25519::from(&secret);
        PublicKey::from(public)
    }
}

prop_compose! {
    pub fn arb_validator()
    (pubkey in arb_pub_key())
    -> Info {
        Info::new(pubkey, Power::try_from(0_i64).unwrap())
    }
}

prop_compose! {
    pub fn fuzz_set(set: Set)
    (val in arb_validator())
    -> Set {
        let mut vals = set.validators().clone();
        if !vals.is_empty() {
            vals[0] = val;
            Set::new(vals, None)
         } else {
            Set::new(vec![val], None)
         }
    }
}
