use serde::{Deserialize, Serialize};
use tendermint_proto::v0_37::types::TxProof as RawTxProof;
use tendermint_proto::Protobuf;

use crate::{merkle, prelude::*, Error, Hash};

/// Merkle proof of the presence of a transaction in the Merkle tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawTxProof", into = "RawTxProof")]
pub struct Proof {
    pub root_hash: Hash,
    pub data: Vec<u8>,
    pub proof: merkle::Proof,
}

impl Protobuf<RawTxProof> for Proof {}

impl TryFrom<RawTxProof> for Proof {
    type Error = Error;

    fn try_from(message: RawTxProof) -> Result<Self, Self::Error> {
        Ok(Self {
            root_hash: message.root_hash.try_into()?,
            data: message.data,
            proof: message.proof.ok_or_else(Error::missing_data)?.try_into()?,
        })
    }
}

impl From<Proof> for RawTxProof {
    fn from(value: Proof) -> Self {
        Self {
            root_hash: value.root_hash.into(),
            data: value.data,
            proof: Some(value.proof.into()),
        }
    }
}
