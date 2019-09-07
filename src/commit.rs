//! Tendermint commits

use crate::{amino_types, block, validator, vote};
use prost::Message;
use signatory::Verifier;
use signatory_dalek::Ed25519Verifier;

use amino_types::vote::CanonicalVote;

/// TODO return Result<(), Error>
/// NOTE: This assumes the vals correspond directly to the votes in the last_commit
pub fn verify_commit(chain_id: &str, last_commit: block::LastCommit, vals: validator::Set) -> bool {
    let precommits_vec = last_commit.precommits.into_vec();
    let vals_vec = vals.into_vec();
    if precommits_vec.len() != vals_vec.len() {
        return false;
    }

    if vals_vec.len() == 0 {
        return false;
    }

    // populate these as we iterate through
    let mut signed_power = 0;
    let mut total_power = 0;

    // populate these from the first non-empty vote
    let mut height = 0;
    let mut round = 0;

    for (val, opt_vote) in vals_vec.into_iter().zip(precommits_vec.into_iter()) {
        let val_power = val.voting_power.value();
        total_power += val_power;

        if let Some(v) = opt_vote {
            if height == 0 {
                height = v.height.value();
                round = v.round;
            }

            if height != v.height.value() || round != v.round {
                return false;
            }

            if v.vote_type != vote::Type::Precommit {
                return false;
            }

            // just skip different block ids
            if v.block_id != last_commit.block_id {
                continue;
            }

            // no validation we can do on the timestamp

            // check the vote matches the validator
            if v.validator_address != val.pub_key.id() {
                return false;
            }

            let mut sign_bytes: Vec<u8> = Vec::new();
            let canonical_vote = CanonicalVote::new(v.to_amino(), chain_id);
            let result = canonical_vote.encode_length_delimited(&mut sign_bytes);
            if let Err(r) = result {
                println!("ERROR {}", r);
                return false;
            }

            // verify the signature
            // TODO: abstract over Ed25519
            let pub_key = val.pub_key.ed25519().unwrap();
            let sig_verifier = Ed25519Verifier::from(&pub_key);
            let result = sig_verifier.verify(&sign_bytes, &v.signature.ed25519().unwrap());
            if let Err(r) = result {
                println!("ERROR {}", r);
                return false;
            }

            signed_power += val_power
        }
    }
    signed_power * 3 > total_power * 2
}
