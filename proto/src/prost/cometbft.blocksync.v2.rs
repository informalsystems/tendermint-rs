/// BlockResponse returns block to the requested
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockResponse {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<super::super::types::v3::Block>,
    #[prost(message, optional, tag = "2")]
    pub ext_commit: ::core::option::Option<super::super::types::v3::ExtendedCommit>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        BlockRequest(super::super::v1::BlockRequest),
        #[prost(message, tag = "2")]
        NoBlockResponse(super::super::v1::NoBlockResponse),
        #[prost(message, tag = "3")]
        BlockResponse(super::BlockResponse),
        #[prost(message, tag = "4")]
        StatusRequest(super::super::v1::StatusRequest),
        #[prost(message, tag = "5")]
        StatusResponse(super::super::v1::StatusResponse),
    }
}
