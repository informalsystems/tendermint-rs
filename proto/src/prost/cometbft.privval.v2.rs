/// SignVoteRequest is a request to sign a vote
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignVoteRequest {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::types::v3::Vote>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
}
/// SignedVoteResponse is a response containing a signed vote or an error
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedVoteResponse {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::types::v3::Vote>,
    #[prost(message, optional, tag = "2")]
    pub error: ::core::option::Option<super::v1::RemoteSignerError>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5, 6, 7, 8")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        PubKeyRequest(super::super::v1::PubKeyRequest),
        #[prost(message, tag = "2")]
        PubKeyResponse(super::super::v1::PubKeyResponse),
        #[prost(message, tag = "3")]
        SignVoteRequest(super::SignVoteRequest),
        #[prost(message, tag = "4")]
        SignedVoteResponse(super::SignedVoteResponse),
        #[prost(message, tag = "5")]
        SignProposalRequest(super::super::v1::SignProposalRequest),
        #[prost(message, tag = "6")]
        SignedProposalResponse(super::super::v1::SignedProposalResponse),
        #[prost(message, tag = "7")]
        PingRequest(super::super::v1::PingRequest),
        #[prost(message, tag = "8")]
        PingResponse(super::super::v1::PingResponse),
    }
}
