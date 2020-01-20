use crate::block::header;
use prost_amino_derive::Message;

#[derive(Clone, Message)]
pub struct Consensus {
    /// Block version
    #[prost_amino(uint64, tag = "1")]
    pub block: u64,

    /// App version
    #[prost_amino(uint64, tag = "2")]
    pub app: u64,
}

impl From<&header::Version> for Consensus {
    fn from(version: &header::Version) -> Self {
        Self {
            block: version.block,
            app: version.app,
        }
    }
}
