#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetAddress {
    #[prost(string, tag="1")]
    pub id: std::string::String,
    #[prost(string, tag="2")]
    pub ip: std::string::String,
    #[prost(uint32, tag="3")]
    pub port: u32,
}
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
pub struct DefaultNodeInfo {
    #[prost(message, optional, tag="1")]
    pub protocol_version: ::std::option::Option<ProtocolVersion>,
    #[prost(string, tag="2")]
    pub default_node_id: std::string::String,
    #[prost(string, tag="3")]
    pub listen_addr: std::string::String,
    #[prost(string, tag="4")]
    pub network: std::string::String,
    #[prost(string, tag="5")]
    pub version: std::string::String,
    #[prost(bytes, tag="6")]
    pub channels: std::vec::Vec<u8>,
    #[prost(string, tag="7")]
    pub moniker: std::string::String,
    #[prost(message, optional, tag="8")]
    pub other: ::std::option::Option<DefaultNodeInfoOther>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DefaultNodeInfoOther {
    #[prost(string, tag="1")]
    pub tx_index: std::string::String,
    #[prost(string, tag="2")]
    pub rpc_address: std::string::String,
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
    #[prost(bytes, tag="3")]
    pub data: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Packet {
    #[prost(oneof="packet::Sum", tags="1, 2, 3")]
    pub sum: ::std::option::Option<packet::Sum>,
}
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
    pub pub_key: ::std::option::Option<super::crypto::PublicKey>,
    #[prost(bytes, tag="2")]
    pub sig: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PexAddrs {
    #[prost(message, repeated, tag="1")]
    pub addrs: ::std::vec::Vec<NetAddress>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof="message::Sum", tags="1, 2")]
    pub sum: ::std::option::Option<message::Sum>,
}
pub mod message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        PexRequest(super::PexRequest),
        #[prost(message, tag="2")]
        PexAddrs(super::PexAddrs),
    }
}
