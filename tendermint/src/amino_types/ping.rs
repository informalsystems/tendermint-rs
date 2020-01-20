use prost_amino_derive::Message;

pub const AMINO_NAME: &str = "tendermint/remotesigner/PingRequest";

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/PingRequest"]
pub struct Request {}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/PingResponse"]
pub struct Response {}
