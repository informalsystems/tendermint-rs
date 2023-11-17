/// Vote is sent when voting for a proposal (or lack thereof).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::types::v1beta3::Vote>,
}
/// HasProposalBlockPart is sent to indicate that a particular proposal block part has been received.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HasProposalBlockPart {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(int32, tag = "3")]
    pub index: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        NewRoundStep(super::super::v1beta1::NewRoundStep),
        #[prost(message, tag = "2")]
        NewValidBlock(super::super::v1beta1::NewValidBlock),
        #[prost(message, tag = "3")]
        Proposal(super::super::v1beta1::Proposal),
        #[prost(message, tag = "4")]
        ProposalPol(super::super::v1beta1::ProposalPol),
        #[prost(message, tag = "5")]
        BlockPart(super::super::v1beta1::BlockPart),
        #[prost(message, tag = "6")]
        Vote(super::Vote),
        #[prost(message, tag = "7")]
        HasVote(super::super::v1beta1::HasVote),
        #[prost(message, tag = "8")]
        VoteSetMaj23(super::super::v1beta1::VoteSetMaj23),
        #[prost(message, tag = "9")]
        VoteSetBits(super::super::v1beta1::VoteSetBits),
        #[prost(message, tag = "10")]
        HasProposalBlockPart(super::HasProposalBlockPart),
    }
}
/// MsgInfo are msgs from the reactor which may update the state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInfo {
    #[prost(message, optional, tag = "1")]
    pub msg: ::core::option::Option<Message>,
    #[prost(string, tag = "2")]
    pub peer_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WalMessage {
    #[prost(oneof = "wal_message::Sum", tags = "1, 2, 3, 4")]
    pub sum: ::core::option::Option<wal_message::Sum>,
}
/// Nested message and enum types in `WALMessage`.
pub mod wal_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        EventDataRoundState(super::super::super::types::v1beta1::EventDataRoundState),
        #[prost(message, tag = "2")]
        MsgInfo(super::MsgInfo),
        #[prost(message, tag = "3")]
        TimeoutInfo(super::super::v1beta1::TimeoutInfo),
        #[prost(message, tag = "4")]
        EndHeight(super::super::v1beta1::EndHeight),
    }
}
/// TimedWALMessage wraps WALMessage and adds Time for debugging purposes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimedWalMessage {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(message, optional, tag = "2")]
    pub msg: ::core::option::Option<WalMessage>,
}
