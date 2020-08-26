use super::validate::{ConsensusMessage, Kind::InvalidHashSize};
use crate::block::parts;
use crate::{
    block,
    error::Error,
    hash,
    hash::{Hash, SHA256_HASH_SIZE},
};
use tendermint_proto::types::BlockId as RawBlockId;
use tendermint_proto::types::CanonicalBlockId as RawCanonicalBlockId;
use tendermint_proto::types::CanonicalPartSetHeader as RawCanonicalPartSetHeader;
use tendermint_proto::types::PartSetHeader as RawPartSetHeader;

/// BlockID
#[derive(Clone, PartialEq)]
pub struct BlockId {
    pub hash: Vec<u8>,
    pub part_set_header: ::std::option::Option<PartSetHeader>,
}

impl From<RawBlockId> for BlockId {
    fn from(value: RawBlockId) -> Self {
        BlockId {
            hash: value.hash,
            part_set_header: match value.part_set_header {
                None => None,
                Some(raw_part_set_header) => Some(PartSetHeader::from(raw_part_set_header)),
            },
        }
    }
}

impl From<BlockId> for RawBlockId {
    fn from(value: BlockId) -> Self {
        RawBlockId {
            hash: value.hash,
            part_set_header: match value.part_set_header {
                None => None,
                Some(part_set_header) => Some(RawPartSetHeader::from(part_set_header)),
            },
        }
    }
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

#[derive(Clone, PartialEq)]
pub struct CanonicalBlockId {
    pub hash: Vec<u8>,
    pub part_set_header: Option<CanonicalPartSetHeader>,
}

impl From<RawCanonicalBlockId> for CanonicalBlockId {
    fn from(value: RawCanonicalBlockId) -> Self {
        CanonicalBlockId {
            hash: value.hash,
            part_set_header: match value.part_set_header {
                None => None,
                Some(raw_part_set_header) => {
                    Some(CanonicalPartSetHeader::from(raw_part_set_header))
                }
            },
        }
    }
}

impl From<CanonicalBlockId> for RawCanonicalBlockId {
    fn from(value: CanonicalBlockId) -> Self {
        RawCanonicalBlockId {
            hash: value.hash,
            part_set_header: match value.part_set_header {
                None => None,
                Some(part_set_header) => Some(part_set_header.into()),
            },
        }
    }
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

/// PartsetHeader
#[derive(Clone, PartialEq)]
pub struct PartSetHeader {
    pub total: i64,
    pub hash: Vec<u8>,
}

impl From<RawPartSetHeader> for PartSetHeader {
    fn from(value: RawPartSetHeader) -> Self {
        PartSetHeader {
            total: value.total as i64,
            hash: value.hash,
        }
    }
}

impl From<PartSetHeader> for RawPartSetHeader {
    fn from(value: PartSetHeader) -> Self {
        RawPartSetHeader {
            total: value.total as u32,
            hash: value.hash,
        }
    }
}

impl PartSetHeader {
    pub fn new(total: i64, hash: Vec<u8>) -> Self {
        PartSetHeader { total: total, hash }
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

#[derive(Clone, PartialEq)]
pub struct CanonicalPartSetHeader {
    pub total: i64,
    pub hash: Vec<u8>,
}

impl From<RawCanonicalPartSetHeader> for CanonicalPartSetHeader {
    fn from(value: RawCanonicalPartSetHeader) -> Self {
        CanonicalPartSetHeader {
            total: value.total as i64,
            hash: value.hash,
        }
    }
}

impl From<CanonicalPartSetHeader> for RawCanonicalPartSetHeader {
    fn from(value: CanonicalPartSetHeader) -> Self {
        RawCanonicalPartSetHeader {
            total: value.total as u32,
            hash: value.hash,
        }
    }
}

impl CanonicalPartSetHeader {
    fn parse_part_set_header(&self) -> Option<block::parts::Header> {
        Hash::new(hash::Algorithm::Sha256, &self.hash)
            .map(|hash| block::parts::Header::new(self.total as u64, hash))
            .ok()
    }
}
