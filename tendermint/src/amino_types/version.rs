use crate::block::*;

// Copied from tendermint_proto::version::Consensus
// and renamed to ConsensusVersion for now
/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusVersion {
    #[prost(uint64, tag = "1")]
    pub block: u64,
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
