//! Block headers

use crate::merkle::simple_hash_from_byte_vectors;
use crate::serializers;
use crate::{account, block, chain, AppHash, Hash, Time};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
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
    pub app_hash: AppHash,

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
        fields_bytes.push(self.chain_id.encode_vec().unwrap());
        fields_bytes.push(self.height.encode_vec().unwrap());
        fields_bytes.push(self.time.encode_vec().unwrap());
        fields_bytes.push(self.last_block_id.unwrap_or_default().encode_vec().unwrap());
        fields_bytes.push(
            self.last_commit_hash
                .unwrap_or_default()
                .encode_vec()
                .unwrap(),
        );
        fields_bytes.push(self.data_hash.unwrap_or_default().encode_vec().unwrap());
        fields_bytes.push(self.validators_hash.encode_vec().unwrap());
        fields_bytes.push(self.next_validators_hash.encode_vec().unwrap());
        fields_bytes.push(self.consensus_hash.encode_vec().unwrap());
        fields_bytes.push(self.app_hash.encode_vec().unwrap());
        fields_bytes.push(
            self.last_results_hash
                .unwrap_or_default()
                .encode_vec()
                .unwrap(),
        );
        fields_bytes.push(self.evidence_hash.unwrap_or_default().encode_vec().unwrap());
        fields_bytes.push(self.proposer_address.encode_vec().unwrap());

        Hash::Sha256(simple_hash_from_byte_vectors(fields_bytes))
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
    ///
    /// If this field is not supplied when deserializing from JSON, it is set
    /// to `Default::default()` for `u64` (i.e. 0).
    #[serde(with = "serializers::from_str", default)]
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
    use super::{Header, Version};
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

    #[test]
    fn empty_header_version_app_field() {
        let json_data = r#"{"block": "11"}"#;
        let version: Version = serde_json::from_str(json_data).unwrap();
        assert_eq!(11, version.block);
        assert_eq!(0, version.app);
    }
}
