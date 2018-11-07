use algorithm::HashAlgorithm;
use block;
use error::Error;
use hash::Hash;

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

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalPartSetHeader {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(int64, tag = "2")]
    pub total: i64,
}
