//! CommitSig within Commit

use crate::{account, Signature, Time};
use serde::de::{self, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Clone, Debug, PartialEq)]
pub enum CommitSig {
    /// no vote was received from a validator.
    BlockIDFlagAbsent {
        /// Validator address
        validator_address: account::Id,
    },
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

/// CommitSig implementation
impl CommitSig {
    /// Helper: Extract validator address, since it's always present (according to ADR-025)
    pub fn validator_address(&self) -> &account::Id {
        match &self {
            CommitSig::BlockIDFlagAbsent { validator_address } => validator_address,
            CommitSig::BlockIDFlagCommit {
                validator_address, ..
            } => validator_address,
            CommitSig::BlockIDFlagNil {
                validator_address, ..
            } => validator_address,
        }
    }
}

impl Serialize for CommitSig {
    fn serialize<S>(
        &self,
        _serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        unimplemented!("CommitSig serialization is not implemented yet")
    }
}

impl<'de> Deserialize<'de> for CommitSig {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct CommitSigVisitor;

        impl<'de> Visitor<'de> for CommitSigVisitor {
            type Value = CommitSig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("CommitSig")
            }

            /// Deserialize map Visitor implementation
            // Implements decision: https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-025-commit.md#decision
            fn visit_map<A>(self, map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                // Instead of manually deserializing the whole struct (cumbersome), we use annotations
                fn option_signature<'de, D>(deserializer: D) -> Result<Option<Signature>, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    Deserialize::deserialize(deserializer).map(|x: Option<_>| x.unwrap_or(None))
                }
                #[derive(Deserialize)]
                struct CommitSigSpec {
                    block_id_flag: u8,
                    validator_address: account::Id,
                    #[serde(default)]
                    timestamp: Option<Time>,
                    #[serde(default, deserialize_with = "option_signature")]
                    signature: Option<Signature>,
                }

                // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
                // into a `Deserializer`, allowing it to be used as the input to T's
                // `Deserialize` implementation. T then deserializes itself using
                // the entries from the map visitor.
                let incoming =
                    CommitSigSpec::deserialize(de::value::MapAccessDeserializer::new(map))?;

                // Validate CommitSig (strict)
                match incoming.block_id_flag {
                    // BlockIDFlagAbsent
                    1 => {
                        if incoming.timestamp.is_some() {
                            Err(A::Error::custom(format!(
                                "timestamp is present for BlockIDFlagAbsent CommitSig {}",
                                incoming.timestamp.unwrap()
                            )))
                        } else {
                            if incoming.signature.is_some() {
                                Err(A::Error::custom(format!(
                                    "signature is present for BlockIDFlagAbsent CommitSig {:?}",
                                    incoming.signature.unwrap()
                                )))
                            } else {
                                Ok(CommitSig::BlockIDFlagAbsent {
                                    validator_address: incoming.validator_address,
                                })
                            }
                        }
                    }
                    // BlockIDFlagCommit
                    2 => {
                        if incoming.timestamp.is_none() {
                            Err(A::Error::custom(
                                "timestamp is null for BlockIDFlagCommit CommitSig",
                            ))
                        } else {
                            if incoming.signature.is_none() {
                                Err(A::Error::custom(
                                    "signature is null for BlockIDFlagCommit CommitSig",
                                ))
                            } else {
                                Ok(CommitSig::BlockIDFlagCommit {
                                    validator_address: incoming.validator_address,
                                    timestamp: incoming.timestamp.unwrap(),
                                    signature: incoming.signature.unwrap(),
                                })
                            }
                        }
                    }
                    // BlockIDFlagNil
                    3 => {
                        if incoming.timestamp.is_none() {
                            Err(A::Error::custom(
                                "timestamp is null for BlockIDFlagNil CommitSig",
                            ))
                        } else {
                            if incoming.signature.is_none() {
                                Err(A::Error::custom(
                                    "signature is null for BlockIDFlagNil CommitSig",
                                ))
                            } else {
                                Ok(CommitSig::BlockIDFlagNil {
                                    validator_address: incoming.validator_address,
                                    timestamp: incoming.timestamp.unwrap(),
                                    signature: incoming.signature.unwrap(),
                                })
                            }
                        }
                    }
                    // Error: unknown CommitSig type
                    e => Err(A::Error::custom(format!(
                        "unknown BlockIdFlag value in CommitSig {}",
                        e
                    ))),
                }
            }
        }

        deserializer.deserialize_map(CommitSigVisitor)
    }
}
