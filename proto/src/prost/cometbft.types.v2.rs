/// ConsensusParams contains consensus critical parameters that determine the
/// validity of blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag = "2")]
    pub evidence: ::core::option::Option<super::v1::EvidenceParams>,
    #[prost(message, optional, tag = "3")]
    pub validator: ::core::option::Option<super::v1::ValidatorParams>,
    #[prost(message, optional, tag = "4")]
    pub version: ::core::option::Option<VersionParams>,
}
/// BlockParams contains limits on the block size.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Max block size, in bytes.
    /// Note: must be greater than 0
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Max gas per block.
    /// Note: must be greater or equal to -1
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}
/// VersionParams contains the ABCI application version.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionParams {
    /// Was named app_version in v1
    #[prost(uint64, tag = "1")]
    pub app: u64,
}
