use super::validate::{ConsensusMessage, ValidationError, ValidationErrorKind::*};
use crate::{
    algorithm::HashAlgorithm,
    block,
    error::Error,
    hash::{Hash, SHA256_HASH_SIZE},
};

#[derive(Clone, PartialEq, Message)]
pub struct BlockId {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(message, tag = "2")]
    pub parts_header: Option<PartsSetHeader>,
}

impl block::ParseId for BlockId {
    fn parse_block_id(&self) -> Result<block::Id, Error> {
        let hash = Hash::new(HashAlgorithm::Sha256, &self.hash)?;
        Ok(block::Id::new(hash))
    }
}

impl ConsensusMessage for BlockId {
    fn validate_basic(&self) -> Result<(), ValidationError> {
        // Hash can be empty in case of POLBlockID in Proposal.
        if !self.hash.is_empty() && self.hash.len() != SHA256_HASH_SIZE {
            return Err(InvalidHashSize.into());
        }
        self.parts_header
            .as_ref()
            .map_or(Ok(()), |psh| psh.validate_basic())
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalBlockId {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(message, tag = "2")]
    pub parts_header: Option<CanonicalPartSetHeader>,
}

impl block::ParseId for CanonicalBlockId {
    fn parse_block_id(&self) -> Result<block::Id, Error> {
        let hash = Hash::new(HashAlgorithm::Sha256, &self.hash)?;
        Ok(block::Id::new(hash))
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct PartsSetHeader {
    #[prost(int64, tag = "1")]
    pub total: i64,
    #[prost(bytes, tag = "2")]
    pub hash: Vec<u8>,
}

impl ConsensusMessage for PartsSetHeader {
    fn validate_basic(&self) -> Result<(), ValidationError> {
        if self.total < 0 {
            return Err(NegativeTotal.into());
        }
        // Hash can be empty in case of POLBlockID.PartsHeader in Proposal.
        if !self.hash.is_empty() && self.hash.len() != SHA256_HASH_SIZE {
            return Err(InvalidHashSize.into());
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalPartSetHeader {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(int64, tag = "2")]
    pub total: i64,
}
