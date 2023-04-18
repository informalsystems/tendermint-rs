/// Vote is sent when voting for a proposal (or lack thereof).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<super::super::types::v3::Vote>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof = "message::Sum", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        NewRoundStep(super::super::v1::NewRoundStep),
        #[prost(message, tag = "2")]
        NewValidBlock(super::super::v1::NewValidBlock),
        #[prost(message, tag = "3")]
        Proposal(super::super::v1::Proposal),
        #[prost(message, tag = "4")]
        ProposalPol(super::super::v1::ProposalPol),
        #[prost(message, tag = "5")]
        BlockPart(super::super::v1::BlockPart),
        #[prost(message, tag = "6")]
        Vote(super::Vote),
        #[prost(message, tag = "7")]
        HasVote(super::super::v1::HasVote),
        #[prost(message, tag = "8")]
        VoteSetMaj23(super::super::v1::VoteSetMaj23),
        #[prost(message, tag = "9")]
        VoteSetBits(super::super::v1::VoteSetBits),
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
        EventDataRoundState(super::super::super::types::v1::EventDataRoundState),
        #[prost(message, tag = "2")]
        MsgInfo(super::MsgInfo),
        #[prost(message, tag = "3")]
        TimeoutInfo(super::super::v1::TimeoutInfo),
        #[prost(message, tag = "4")]
        EndHeight(super::super::v1::EndHeight),
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
