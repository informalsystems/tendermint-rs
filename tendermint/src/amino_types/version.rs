use crate::block::*;

#[derive(Clone, Message)]
pub struct ConsensusVersion {
    /// Block version
    #[prost(uint64, tag = "1")]
    pub block: u64,

    /// App version
    #[prost(uint64, tag = "2")]
    pub app: u64,
}

impl From<&header::Version> for ConsensusVersion {
    fn from(version: &header::Version) -> Self {
        ConsensusVersion {
            block: version.block,
            app: version.app,
        }
    }
}
