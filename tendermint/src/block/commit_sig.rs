//! CommitSig within Commit

use crate::serializers::BlockIDFlag;
use crate::serializers::RawCommitSig;
use crate::{account, Signature, Time};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(try_from = "RawCommitSig")]
pub enum CommitSig {
    /// no vote was received from a validator.
    // <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/260
    BlockIDFlagAbsent,
    // === Real code after compatibility issue is resolved
    /*
    BlockIDFlagAbsent {
        /// Validator address
        validator_address: account::Id,
    },
    */
    // >>> end of real code
    /// voted for the Commit.BlockID.
    BlockIDFlagCommit {
        /// Validator address
        validator_address: account::Id,
        /// Timestamp of vote
        timestamp: Time,
        /// Signature of vote
        signature: Signature,
    },
    /// voted for nil.
    BlockIDFlagNil {
        /// Validator address
        validator_address: account::Id,
        /// Timestamp of vote
        timestamp: Time,
        /// Signature of vote
        signature: Signature,
    },
}

impl TryFrom<RawCommitSig> for CommitSig {
    type Error = &'static str;

    fn try_from(value: RawCommitSig) -> Result<Self, Self::Error> {
        // Validate CommitSig (strict)
        match value.block_id_flag {
            BlockIDFlag::BlockIDFlagAbsent => {
                if value.timestamp.is_some() {
                    // <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/259
                    if value.timestamp.unwrap()
                        != Time::parse_from_rfc3339("0001-01-01T00:00:00Z").unwrap()
                    {
                        return Err("timestamp is present for BlockIDFlagAbsent CommitSig");
                    }
                    // === Real code after compatibility issue is resolved
                    /*
                    return Err("timestamp is present for BlockIDFlagAbsent CommitSig");
                    */
                    // >>> end of real code
                }
                if value.signature.is_some() {
                    return Err("signature is present for BlockIDFlagAbsent CommitSig");
                }
                // <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/260
                Ok(CommitSig::BlockIDFlagAbsent)
                // === Real code after compatibility issue is resolved
                /*
                Ok(CommitSig::BlockIDFlagAbsent {
                    validator_address: value.validator_address,
                })
                */
                // >>> end of real code
            }
            BlockIDFlag::BlockIDFlagCommit => {
                if value.timestamp.is_none() {
                    Err("timestamp is missing for BlockIDFlagCommit CommitSig")
                } else if value.signature.is_none() {
                    Err("signature is null for BlockIDFlagCommit CommitSig")
                } else {
                    Ok(CommitSig::BlockIDFlagCommit {
                        // <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/260
                        validator_address: value.validator_address.unwrap(),
                        // === Real code after compatibility issue is resolved
                        /*
                        validator_address: value.validator_address,
                        */
                        // >>> end of real code
                        timestamp: value.timestamp.unwrap(),
                        signature: value.signature.unwrap(),
                    })
                }
            }
            BlockIDFlag::BlockIDFlagNil => {
                if value.timestamp.is_none() {
                    Err("timestamp is null for BlockIDFlagNil CommitSig")
                } else if value.signature.is_none() {
                    Err("signature is null for BlockIDFlagNil CommitSig")
                } else {
                    Ok(CommitSig::BlockIDFlagNil {
                        // <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/260
                        validator_address: value.validator_address.unwrap(),
                        // === Real code after compatibility issue is resolved
                        /*
                        validator_address: value.validator_address,
                        */
                        // >>> end of real code
                        timestamp: value.timestamp.unwrap(),
                        signature: value.signature.unwrap(),
                    })
                }
            }
        }
    }
}
