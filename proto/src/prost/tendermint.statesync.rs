#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof="message::Sum", tags="1, 2, 3, 4")]
    pub sum: ::std::option::Option<message::Sum>,
}
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
    #[prost(bytes, tag="4")]
    pub hash: std::vec::Vec<u8>,
    #[prost(bytes, tag="5")]
    pub metadata: std::vec::Vec<u8>,
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
    #[prost(bytes, tag="4")]
    pub chunk: std::vec::Vec<u8>,
    #[prost(bool, tag="5")]
    pub missing: bool,
}
