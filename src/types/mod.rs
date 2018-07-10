
extern crate prost;

use prost::Message;

mod heartbeat;
mod proposal;
mod vote;

#[derive(Clone, PartialEq, Message)]
pub struct PartsSetHeader {
    #[prost(sint64, tag="1")]
    total: i64,
    #[prost(bytes, tag="2")]
    hash: Vec<u8>,
}


#[derive(Clone, PartialEq, Message)]
pub struct BlockID {
    #[prost(bytes, tag="1")]
    hash: Vec<u8>,
    #[prost(message, tag="2")]
    parts_header: Option<PartsSetHeader>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Time {
    #[prost(sfixed64, tag="1")]
    pub seconds: i64,
    #[prost(sfixed32, tag="2")]
    pub nanos: i32,
}
pub trait TendermintSign {
    fn cannonicalize(self, chain_id: &str) -> String;
}

