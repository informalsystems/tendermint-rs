#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProtocolVersion {
    #[prost(uint64, tag="1")]
    pub p2p: u64,
    #[prost(uint64, tag="2")]
    pub block: u64,
    #[prost(uint64, tag="3")]
    pub app: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeInfo {
    #[prost(message, optional, tag="1")]
    pub protocol_version: ::core::option::Option<ProtocolVersion>,
    #[prost(string, tag="2")]
    pub node_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub listen_addr: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub network: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub version: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="6")]
    pub channels: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub moniker: ::prost::alloc::string::String,
    #[prost(message, optional, tag="8")]
    pub other: ::core::option::Option<NodeInfoOther>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeInfoOther {
    #[prost(string, tag="1")]
    pub tx_index: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub rpc_address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PeerInfo {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="2")]
    pub address_info: ::prost::alloc::vec::Vec<PeerAddressInfo>,
    #[prost(message, optional, tag="3")]
    pub last_connected: ::core::option::Option<super::super::google::protobuf::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PeerAddressInfo {
    #[prost(string, tag="1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub last_dial_success: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(message, optional, tag="3")]
    pub last_dial_failure: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(uint32, tag="4")]
    pub dial_failures: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketPing {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketPong {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketMsg {
    #[prost(int32, tag="1")]
    pub channel_id: i32,
    #[prost(bool, tag="2")]
    pub eof: bool,
    #[prost(bytes="vec", tag="3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Packet {
    #[prost(oneof="packet::Sum", tags="1, 2, 3")]
    pub sum: ::core::option::Option<packet::Sum>,
}
/// Nested message and enum types in `Packet`.
pub mod packet {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        PacketPing(super::PacketPing),
        #[prost(message, tag="2")]
        PacketPong(super::PacketPong),
        #[prost(message, tag="3")]
        PacketMsg(super::PacketMsg),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthSigMessage {
    #[prost(message, optional, tag="1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(bytes="vec", tag="2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexAddress {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub ip: ::prost::alloc::string::String,
    #[prost(uint32, tag="3")]
    pub port: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexResponse {
    #[prost(message, repeated, tag="1")]
    pub addresses: ::prost::alloc::vec::Vec<PexAddress>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexAddressV2 {
    #[prost(string, tag="1")]
    pub url: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexRequestV2 {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexResponseV2 {
    #[prost(message, repeated, tag="1")]
    pub addresses: ::prost::alloc::vec::Vec<PexAddressV2>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexMessage {
    #[prost(oneof="pex_message::Sum", tags="1, 2, 3, 4")]
    pub sum: ::core::option::Option<pex_message::Sum>,
}
/// Nested message and enum types in `PexMessage`.
pub mod pex_message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        PexRequest(super::PexRequest),
        #[prost(message, tag="2")]
        PexResponse(super::PexResponse),
        #[prost(message, tag="3")]
        PexRequestV2(super::PexRequestV2),
        #[prost(message, tag="4")]
        PexResponseV2(super::PexResponseV2),
    }
}
