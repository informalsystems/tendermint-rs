#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof="message::Sum", tags="1, 2, 3, 4, 5, 6, 7, 8")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        SnapshotsRequest(super::SnapshotsRequest),
        #[prost(message, tag="2")]
        SnapshotsResponse(super::SnapshotsResponse),
        #[prost(message, tag="3")]
        ChunkRequest(super::ChunkRequest),
        #[prost(message, tag="4")]
        ChunkResponse(super::ChunkResponse),
        #[prost(message, tag="5")]
        LightBlockRequest(super::LightBlockRequest),
        #[prost(message, tag="6")]
        LightBlockResponse(super::LightBlockResponse),
        #[prost(message, tag="7")]
        ParamsRequest(super::ParamsRequest),
        #[prost(message, tag="8")]
        ParamsResponse(super::ParamsResponse),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotsRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SnapshotsResponse {
    #[prost(uint64, tag="1")]
    pub height: u64,
    #[prost(uint32, tag="2")]
    pub format: u32,
    #[prost(uint32, tag="3")]
    pub chunks: u32,
    #[prost(bytes="vec", tag="4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="5")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChunkRequest {
    #[prost(uint64, tag="1")]
    pub height: u64,
    #[prost(uint32, tag="2")]
    pub format: u32,
    #[prost(uint32, tag="3")]
    pub index: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChunkResponse {
    #[prost(uint64, tag="1")]
    pub height: u64,
    #[prost(uint32, tag="2")]
    pub format: u32,
    #[prost(uint32, tag="3")]
    pub index: u32,
    #[prost(bytes="vec", tag="4")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="5")]
    pub missing: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightBlockRequest {
    #[prost(uint64, tag="1")]
    pub height: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightBlockResponse {
    #[prost(message, optional, tag="1")]
    pub light_block: ::core::option::Option<super::types::LightBlock>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsRequest {
    #[prost(uint64, tag="1")]
    pub height: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParamsResponse {
    #[prost(uint64, tag="1")]
    pub height: u64,
    #[prost(message, optional, tag="2")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
}
