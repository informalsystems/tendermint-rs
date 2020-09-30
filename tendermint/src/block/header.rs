//! Block headers

use crate::merkle::simple_hash_from_byte_vectors;
use crate::serializers;
use crate::{account, block, chain, Hash, Time};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tendermint_proto::types::BlockId;
use tendermint_proto::version::Consensus as RawConsensusVersion;
use tendermint_proto::DomainType;

/// Block `Header` values contain metadata about the block and about the
/// consensus, as well as commitments to the data in the current block, the
/// previous block, and the results returned by the application.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#header>
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Header {
    /// Header version
    pub version: Version,

    /// Chain ID
    pub chain_id: chain::Id,

    /// Current block height
    pub height: block::Height,

    /// Current timestamp
    pub time: Time,

    /// Previous block info
    #[serde(deserialize_with = "serializers::parse_non_empty_block_id")]
    pub last_block_id: Option<block::Id>,

    /// Commit from validators from the last block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub last_commit_hash: Option<Hash>,

    /// Merkle root of transaction hashes
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub data_hash: Option<Hash>,

    /// Validators for the current block
    pub validators_hash: Hash,

    /// Validators for the next block
    pub next_validators_hash: Hash,

    /// Consensus params for the current block
    pub consensus_hash: Hash,

    /// State after txs from the previous block
    #[serde(with = "serializers::bytes::hexstring")]
    pub app_hash: Vec<u8>,

    /// Root hash of all results from the txs from the previous block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub last_results_hash: Option<Hash>,

    /// Hash of evidence included in the block
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub evidence_hash: Option<Hash>,

    /// Original proposer of the block
    pub proposer_address: account::Id,
}

impl Header {
    /// Hash this header
    pub fn hash(&self) -> Hash {
        // Note that if there is an encoding problem this will
        // panic (as the golang code would):
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/block.go#L393
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/encoding_helper.go#L9:6

        let mut fields_bytes: Vec<Vec<u8>> = Vec::with_capacity(14);
        fields_bytes.push(self.version.encode_vec().unwrap());
        fields_bytes.push(encode_to_vec(&self.chain_id.to_string()));
        fields_bytes.push(encode_to_vec(&self.height.value()));
        fields_bytes.push(self.time.encode_vec().unwrap());
        fields_bytes.push(encode_optional(
            &self
                .last_block_id
                .as_ref()
                .map(|id| BlockId::try_from(id.clone()).unwrap()),
        ));
        fields_bytes.push(encode_optional(
            &self
                .last_commit_hash
                .as_ref()
                .map(|hash| hash.as_bytes().to_vec()),
        ));
        fields_bytes.push(encode_optional(
            &self.data_hash.as_ref().map(|hash| hash.as_bytes().to_vec()),
        ));
        fields_bytes.push(encode_to_vec(&self.validators_hash.as_bytes().to_vec()));
        fields_bytes.push(encode_to_vec(
            &self.next_validators_hash.as_bytes().to_vec(),
        ));
        fields_bytes.push(encode_to_vec(&self.consensus_hash.as_bytes().to_vec()));
        fields_bytes.push(encode_to_vec(&self.app_hash));
        fields_bytes.push(encode_optional(
            &self
                .last_results_hash
                .as_ref()
                .map(|hash| hash.as_bytes().to_vec()),
        ));
        fields_bytes.push(encode_optional(
            &self
                .evidence_hash
                .as_ref()
                .map(|hash| hash.as_bytes().to_vec()),
        ));
        fields_bytes.push(encode_to_vec(&self.proposer_address.as_bytes().to_vec()));

        Hash::Sha256(simple_hash_from_byte_vectors(fields_bytes))
    }
}

fn encode_to_vec<T: prost::Message>(val: &T) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    prost::Message::encode(val, &mut buf).map(|_| buf).unwrap()
}

fn encode_optional<T: prost::Message>(val: &Option<T>) -> Vec<u8> {
    match val {
        Some(inner) => encode_to_vec(inner),
        None => vec![],
    }
}

/// `Version` contains the protocol version for the blockchain and the
/// application.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#version>
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Version {
    /// Block version
    #[serde(with = "serializers::from_str")]
    pub block: u64,

    /// App version
    #[serde(with = "serializers::from_str")]
    pub app: u64,
}

impl DomainType<RawConsensusVersion> for Version {}

impl TryFrom<RawConsensusVersion> for Version {
    type Error = anomaly::BoxError;

    fn try_from(value: RawConsensusVersion) -> Result<Self, Self::Error> {
        Ok(Version {
            block: value.block,
            app: value.app,
        })
    }
}

impl From<Version> for RawConsensusVersion {
    fn from(value: Version) -> Self {
        RawConsensusVersion {
            block: value.block,
            app: value.app,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Header;
    use crate::hash::Algorithm;
    use crate::test::test_serialization_roundtrip;
    use crate::Hash;

    #[test]
    fn serialization_roundtrip() {
        let json_data = include_str!("../../tests/support/serialization/block/header.json");
        test_serialization_roundtrip::<Header>(json_data);
    }

    #[test]
    fn header_hashing() {
        let expected_hash = Hash::from_hex_upper(
            Algorithm::Sha256,
            "F30A71F2409FB15AACAEDB6CC122DFA2525BEE9CAE521721B06BFDCA291B8D56",
        )
        .unwrap();
        let header: Header = serde_json::from_str(include_str!(
            "../../tests/support/serialization/block/header_with_known_hash.json"
        ))
        .unwrap();
        assert_eq!(expected_hash, header.hash());
    }
}
