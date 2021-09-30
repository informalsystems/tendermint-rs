//! Block headers

use crate::merkle::simple_hash_from_byte_vectors;
use crate::prelude::*;
use crate::{account, block, chain, AppHash, Error, Hash, Time};
use core::convert::{TryFrom, TryInto};
use serde::{Deserialize, Serialize};
use tendermint_proto::types::Header as RawHeader;
use tendermint_proto::version::Consensus as RawConsensusVersion;
use tendermint_proto::Protobuf;

/// Block `Header` values contain metadata about the block and about the
/// consensus, as well as commitments to the data in the current block, the
/// previous block, and the results returned by the application.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#header>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawHeader", into = "RawHeader")]
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
    pub last_block_id: Option<block::Id>,

    /// Commit from validators from the last block
    pub last_commit_hash: Option<Hash>,

    /// Merkle root of transaction hashes
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
    pub last_results_hash: Option<Hash>,

    /// Hash of evidence included in the block
    pub evidence_hash: Option<Hash>,

    /// Original proposer of the block
    pub proposer_address: account::Id,
}

impl Protobuf<RawHeader> for Header {}

impl TryFrom<RawHeader> for Header {
    type Error = Error;

    fn try_from(value: RawHeader) -> Result<Self, Self::Error> {
        // If last block id is unfilled, it is considered nil by Go.
        let last_block_id = value
            .last_block_id
            .map(TryInto::try_into)
            .transpose()?
            .filter(|l| l != &block::Id::default());
        let last_commit_hash = if value.last_commit_hash.is_empty() {
            None
        } else {
            Some(value.last_commit_hash.try_into()?)
        };
        let last_results_hash = if value.last_results_hash.is_empty() {
            None
        } else {
            Some(value.last_results_hash.try_into()?)
        };
        let height: block::Height = value.height.try_into()?;

        // Todo: fix domain logic
        //if last_block_id.is_none() && height.value() != 1 {
        //    return Err(Kind::InvalidHeader.context("last_block_id is null on non-first
        // height").into());
        //}
        if last_block_id.is_some() && height.value() == 1 {
            return Err(Error::invalid_first_header());
        }
        //if last_commit_hash.is_none() && height.value() != 1 {
        //    return Err(Kind::InvalidHeader.context("last_commit_hash is null on non-first
        // height").into());
        //}
        //if height.value() == 1 && last_commit_hash.is_some() &&
        // last_commit_hash.as_ref().unwrap() != simple_hash_from_byte_vectors(Vec::new()) {
        //    return Err(Kind::InvalidFirstHeader.context("last_commit_hash is not empty Merkle tree
        // on first height").into());
        //}
        //if last_results_hash.is_none() && height.value() != 1 {
        //    return Err(Kind::InvalidHeader.context("last_results_hash is null on non-first
        // height").into());
        //}
        //if last_results_hash.is_some() && height.value() == 1 {
        //    return Err(Kind::InvalidFirstHeader.context("last_results_hash is not ull on first
        // height").into());
        //}
        Ok(Header {
            version: value.version.ok_or_else(Error::missing_version)?.into(),
            chain_id: value.chain_id.try_into()?,
            height,
            time: value
                .time
                .ok_or_else(Error::missing_timestamp)?
                .try_into()?,
            last_block_id,
            last_commit_hash,
            data_hash: if value.data_hash.is_empty() {
                None
            } else {
                Some(value.data_hash.try_into()?)
            },
            validators_hash: value.validators_hash.try_into()?,
            next_validators_hash: value.next_validators_hash.try_into()?,
            consensus_hash: value.consensus_hash.try_into()?,
            app_hash: value.app_hash.try_into()?,
            last_results_hash,
            evidence_hash: if value.evidence_hash.is_empty() {
                None
            } else {
                Some(value.evidence_hash.try_into()?)
            }, // Todo: Is it illegal to have evidence of wrongdoing in the first block?
            proposer_address: value.proposer_address.try_into()?,
        })
    }
}

impl From<Header> for RawHeader {
    fn from(value: Header) -> Self {
        RawHeader {
            version: Some(value.version.into()),
            chain_id: value.chain_id.into(),
            height: value.height.into(),
            time: Some(value.time.into()),
            last_block_id: value.last_block_id.map(Into::into),
            last_commit_hash: value.last_commit_hash.unwrap_or_default().into(),
            data_hash: value.data_hash.unwrap_or_default().into(),
            validators_hash: value.validators_hash.into(),
            next_validators_hash: value.next_validators_hash.into(),
            consensus_hash: value.consensus_hash.into(),
            app_hash: value.app_hash.into(),
            last_results_hash: value.last_results_hash.unwrap_or_default().into(),
            evidence_hash: value.evidence_hash.unwrap_or_default().into(),
            proposer_address: value.proposer_address.into(),
        }
    }
}

impl Header {
    /// Hash this header
    pub fn hash(&self) -> Hash {
        // Note that if there is an encoding problem this will
        // panic (as the golang code would):
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/block.go#L393
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/encoding_helper.go#L9:6

        let fields_bytes = vec![
            self.version.encode_vec().unwrap(),
            self.chain_id.encode_vec().unwrap(),
            self.height.encode_vec().unwrap(),
            self.time.encode_vec().unwrap(),
            self.last_block_id.unwrap_or_default().encode_vec().unwrap(),
            self.last_commit_hash
                .unwrap_or_default()
                .encode_vec()
                .unwrap(),
            self.data_hash.unwrap_or_default().encode_vec().unwrap(),
            self.validators_hash.encode_vec().unwrap(),
            self.next_validators_hash.encode_vec().unwrap(),
            self.consensus_hash.encode_vec().unwrap(),
            self.app_hash.encode_vec().unwrap(),
            self.last_results_hash
                .unwrap_or_default()
                .encode_vec()
                .unwrap(),
            self.evidence_hash.unwrap_or_default().encode_vec().unwrap(),
            self.proposer_address.encode_vec().unwrap(),
        ];

        Hash::Sha256(simple_hash_from_byte_vectors(fields_bytes))
    }
}

/// `Version` contains the protocol version for the blockchain and the
/// application.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#version>
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    /// Block version
    pub block: u64,

    /// App version
    pub app: u64,
}

impl Protobuf<RawConsensusVersion> for Version {}

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
