use super::validate::{ConsensusMessage, Kind::InvalidHashSize};
use crate::block::parts;
use crate::{
    block,
    error::Error,
    hash,
    hash::{Hash, SHA256_HASH_SIZE},
};

// Copied from tendermint_proto::types::BlockId
/// BlockID
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub part_set_header: ::std::option::Option<PartSetHeader>,
}

impl BlockId {
    pub fn new(hash: Vec<u8>, part_set_header: Option<PartSetHeader>) -> Self {
        BlockId {
            hash,
            part_set_header,
        }
    }
}

impl block::ParseId for BlockId {
    fn parse_block_id(&self) -> Result<block::Id, Error> {
        let hash = Hash::new(hash::Algorithm::Sha256, &self.hash)?;
        let part_set_header = self
            .part_set_header
            .as_ref()
            .and_then(PartSetHeader::parse_part_set_header);
        Ok(block::Id::new(hash, part_set_header))
    }
}

impl From<&block::Id> for BlockId {
    fn from(bid: &block::Id) -> Self {
        let bid_hash = bid.hash.as_bytes();
        BlockId::new(
            bid_hash.to_vec(),
            bid.parts.as_ref().map(PartSetHeader::from),
        )
    }
}

impl ConsensusMessage for BlockId {
    fn validate_basic(&self) -> Result<(), Error> {
        // Hash can be empty in case of POLBlockID in Proposal.
        if !self.hash.is_empty() && self.hash.len() != SHA256_HASH_SIZE {
            return Err(InvalidHashSize.into());
        }
        self.part_set_header
            .as_ref()
            .map_or(Ok(()), ConsensusMessage::validate_basic)
    }
}

// Copied from tendermint_proto::types::CanonicalBlockId
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalBlockId {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub part_set_header: ::std::option::Option<CanonicalPartSetHeader>,
}

impl block::ParseId for CanonicalBlockId {
    fn parse_block_id(&self) -> Result<block::Id, Error> {
        let hash = Hash::new(hash::Algorithm::Sha256, &self.hash)?;
        let part_set_header = self
            .part_set_header
            .as_ref()
            .and_then(CanonicalPartSetHeader::parse_part_set_header);
        Ok(block::Id::new(hash, part_set_header))
    }
}

// Copied from tendermint_proto::types::PartSetHeader
/// PartsetHeader
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes, tag = "2")]
    pub hash: Vec<u8>,
}

impl PartSetHeader {
    pub fn new(total: i64, hash: Vec<u8>) -> Self {
        PartSetHeader {
            total: total as u32,
            hash,
        }
    }
}

impl From<&parts::Header> for PartSetHeader {
    fn from(parts: &parts::Header) -> Self {
        PartSetHeader::new(parts.total as i64, parts.hash.as_bytes().to_vec())
    }
}

impl PartSetHeader {
    fn parse_part_set_header(&self) -> Option<block::parts::Header> {
        Hash::new(hash::Algorithm::Sha256, &self.hash)
            .map(|hash| block::parts::Header::new(self.total as u64, hash))
            .ok()
    }
}

impl ConsensusMessage for PartSetHeader {
    fn validate_basic(&self) -> Result<(), Error> {
        // Hash can be empty in case of POLBlockID.PartsHeader in Proposal.
        if !self.hash.is_empty() && self.hash.len() != SHA256_HASH_SIZE {
            return Err(InvalidHashSize.into());
        }
        Ok(())
    }
}

// Copied from tendermint_proto::types::CanonicalPartSetHeader
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalPartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes, tag = "2")]
    pub hash: Vec<u8>,
}

impl CanonicalPartSetHeader {
    fn parse_part_set_header(&self) -> Option<block::parts::Header> {
        Hash::new(hash::Algorithm::Sha256, &self.hash)
            .map(|hash| block::parts::Header::new(self.total as u64, hash))
            .ok()
    }
}
