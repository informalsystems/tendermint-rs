use super::validate::{ConsensusMessage, ValidationError, ValidationErrorKind};
use crate::block::parts;
use crate::{
    block,
    error::Error,
    hash,
    hash::{Hash, SHA256_HASH_SIZE},
};
use prost_amino_derive::Message;
use std::convert::TryFrom;

#[derive(Clone, PartialEq, Message)]
pub struct BlockId {
    #[prost_amino(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost_amino(message, tag = "2")]
    pub parts_header: Option<PartsSetHeader>,
}

impl BlockId {
    pub fn new(hash: Vec<u8>, parts_header: Option<PartsSetHeader>) -> Self {
        Self { hash, parts_header }
    }
}

impl block::ParseId for BlockId {
    fn parse_block_id(&self) -> Result<block::Id, Error> {
        let hash = Hash::new(hash::Algorithm::Sha256, &self.hash)?;
        let parts_header = self
            .parts_header
            .as_ref()
            .and_then(PartsSetHeader::parse_parts_header);
        Ok(block::Id::new(hash, parts_header))
    }
}

impl From<&block::Id> for BlockId {
    fn from(bid: &block::Id) -> Self {
        let bid_hash = bid.hash.as_bytes();
        Self::new(
            bid_hash.to_vec(),
            bid.parts.as_ref().map(PartsSetHeader::from),
        )
    }
}

impl ConsensusMessage for BlockId {
    fn validate_basic(&self) -> Result<(), ValidationError> {
        // Hash can be empty in case of POLBlockID in Proposal.
        if !self.hash.is_empty() && self.hash.len() != SHA256_HASH_SIZE {
            return Err(ValidationErrorKind::InvalidHashSize.into());
        }
        self.parts_header
            .as_ref()
            .map_or(Ok(()), ConsensusMessage::validate_basic)
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Canonical {
    #[prost_amino(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost_amino(message, tag = "2")]
    pub parts_header: Option<CanonicalPartSetHeader>,
}

impl block::ParseId for Canonical {
    fn parse_block_id(&self) -> Result<block::Id, Error> {
        let hash = Hash::new(hash::Algorithm::Sha256, &self.hash)?;
        let parts_header = self
            .parts_header
            .as_ref()
            .and_then(CanonicalPartSetHeader::parse_parts_header);
        Ok(block::Id::new(hash, parts_header))
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct PartsSetHeader {
    #[prost_amino(int64, tag = "1")]
    pub total: i64,
    #[prost_amino(bytes, tag = "2")]
    pub hash: Vec<u8>,
}

impl PartsSetHeader {
    pub fn new(total: i64, hash: Vec<u8>) -> Self {
        Self { total, hash }
    }

    fn parse_parts_header(&self) -> Option<block::parts::Header> {
        Hash::new(hash::Algorithm::Sha256, &self.hash)
            .map(|hash| block::parts::Header::new(self.total as u64, hash))
            .ok()
    }
}

// TODO(xla): Rework into TryFrom.
#[allow(clippy::fallible_impl_from)]
impl From<&parts::Header> for PartsSetHeader {
    fn from(parts: &parts::Header) -> Self {
        Self::new(
            i64::try_from(parts.total).expect("overflow"),
            parts.hash.as_bytes().to_vec(),
        )
    }
}

impl ConsensusMessage for PartsSetHeader {
    fn validate_basic(&self) -> Result<(), ValidationError> {
        if self.total < 0 {
            return Err(ValidationErrorKind::NegativeTotal.into());
        }
        // Hash can be empty in case of POLBlockID.PartsHeader in Proposal.
        if !self.hash.is_empty() && self.hash.len() != SHA256_HASH_SIZE {
            return Err(ValidationErrorKind::InvalidHashSize.into());
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalPartSetHeader {
    #[prost_amino(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost_amino(int64, tag = "2")]
    pub total: i64,
}

impl CanonicalPartSetHeader {
    fn parse_parts_header(&self) -> Option<block::parts::Header> {
        Hash::new(hash::Algorithm::Sha256, &self.hash)
            .map(|hash| block::parts::Header::new(self.total as u64, hash))
            .ok()
    }
}
