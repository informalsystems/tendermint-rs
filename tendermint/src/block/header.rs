//! Block headers

use crate::amino_types::{message::AminoMessage, BlockId};
use crate::merkle::simple_hash_from_byte_vectors;
use crate::serializers;
use crate::{account, block, chain, Hash, Time};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use tendermint_proto::types::BlockId as RawBlockId;
use tendermint_proto::version::Consensus as RawConsensusVersion;

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

        let raw_consensus_version: RawConsensusVersion = self.version.clone().into();

        let mut fields_bytes: Vec<Vec<u8>> = Vec::with_capacity(16);
        fields_bytes.push(AminoMessage::bytes_vec(&raw_consensus_version));
        fields_bytes.push(encode_bytes(self.chain_id.as_bytes()));
        fields_bytes.push(encode_varint(self.height.value()));
        fields_bytes.push(AminoMessage::bytes_vec(&Timestamp::from(
            self.time.to_system_time().unwrap(),
        )));
        match &self.last_block_id {
            None => {
                let raw_block_id: RawBlockId = BlockId::new(vec![], None).into();
                AminoMessage::bytes_vec(&raw_block_id);
            }
            Some(id) => {
                let raw_block_id: RawBlockId = BlockId::from(id).into();
                AminoMessage::bytes_vec(&raw_block_id);
            }
        }
        fields_bytes.push(self.last_commit_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(self.data_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(encode_hash(&self.validators_hash));
        fields_bytes.push(encode_hash(&self.next_validators_hash));
        fields_bytes.push(encode_hash(&self.consensus_hash));
        fields_bytes.push(encode_bytes(&self.app_hash));
        fields_bytes.push(self.last_results_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(self.evidence_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(encode_bytes(self.proposer_address.as_bytes()));

        Hash::Sha256(simple_hash_from_byte_vectors(fields_bytes))
    }
}

fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let bytes_len = bytes.len();
    if bytes_len > 0 {
        let mut encoded = vec![];
        prost::encode_length_delimiter(bytes_len, &mut encoded).unwrap();
        encoded.append(&mut bytes.to_vec());
        encoded
    } else {
        vec![]
    }
}

fn encode_hash(hash: &Hash) -> Vec<u8> {
    encode_bytes(hash.as_bytes())
}

fn encode_varint(val: u64) -> Vec<u8> {
    let mut val_enc = vec![];
    prost::encoding::encode_varint(val, &mut val_enc);
    val_enc
}

/// `Version` contains the protocol version for the blockchain and the
/// application.
///
/// When deserializing from JSON, if the `app` field is not supplied it is
/// automatically set to 0.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#version>
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Version {
    /// Block version
    #[serde(with = "serializers::from_str")]
    pub block: u64,

    /// App version
    #[serde(
        with = "serializers::from_str",
        default = "Version::default_app_version"
    )]
    pub app: u64,
}

impl Version {
    // We explicitly set the app version to 0 if it's not supplied in the JSON.
    fn default_app_version() -> u64 {
        0
    }
}

impl From<RawConsensusVersion> for Version {
    fn from(value: RawConsensusVersion) -> Self {
        Version {
            block: value.block,
            app: value.app,
        }
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
    use super::{Header, Version};
    use crate::test::test_serialization_roundtrip;

    #[test]
    fn serialization_roundtrip() {
        let json_data = include_str!("../../tests/support/serialization/block/header.json");
        test_serialization_roundtrip::<Header>(json_data);
    }

    #[test]
    fn empty_header_version_app_field() {
        let json_data = r#"{"block": "11"}"#;
        let version: Version = serde_json::from_str(json_data).unwrap();
        assert_eq!(11, version.block);
        assert_eq!(0, version.app);
    }
}
