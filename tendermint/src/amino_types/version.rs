use crate::block::header;

#[derive(Clone, Message)]
pub struct Consensus {
    /// Block version
    #[prost(uint64, tag = "1")]
    pub block: u64,

    /// App version
    #[prost(uint64, tag = "2")]
    pub app: u64,
}

impl From<&header::Version> for Consensus {
    fn from(version: &header::Version) -> Self {
        Consensus {
            block: version.block,
            app: version.app,
        }
    }
}
