use crate::height::arb_height;
use proptest::prelude::*;
use std::convert::TryFrom;
use tendermint::block::{Commit, CommitSig};

use tendermint::{
    signature::{Ed25519Signature, ED25519_SIGNATURE_SIZE},
    validator::Set,
    Signature,
};

prop_compose! {
    pub fn fuzz_commit(c: Commit)
    (
        height in arb_height(),
        // sig in arb_signature()
    )
    -> Commit {
        let mut commit: Commit = c.clone();
        commit.height = height;
        // Below code won't work right now because of the restrictions on accessible data fields on CommitSig
        // but since we are planning to move these pbt generators close to the respective data structures
        // this code will work there :)
        // if !commit.signatures.is_empty() {
        //     if commit.signatures[0].is_commit() {
        //          commit.signatures[0] = CommitSig::BlockIDFlagCommit {
        //              validator_address: arb_id,
        //              timestamp: arb_time,
        //              signature: sig,
        //         };
        //     }
        //     if commit.signatures[0].is_nil() {
        //         commit.signatures[0] = CommitSig::BlockIDFlagNil {
        //              validator_address: arb_id,
        //              timestamp: arb_time,
        //              signature: sig,
        //         };
        //     }
        // }
        commit
    }
}

prop_compose! {
    pub fn arb_signature()
    (
        bytes in prop::array::uniform32(0u8..),
    )
    -> Signature {
        Signature::Ed25519(
                Ed25519Signature::try_from(&bytes[..]).unwrap()
            )
    }
}
