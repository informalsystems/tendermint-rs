use super::compute_prefix;
use once_cell::sync::Lazy;
use prost_amino_derive::Message;

pub const AMINO_NAME: &str = "tendermint/remotesigner/PingRequest";
pub static AMINO_PREFIX: Lazy<Vec<u8>> = Lazy::new(|| compute_prefix(AMINO_NAME));

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/PingRequest"]
pub struct PingRequest {}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/PingResponse"]
pub struct PingResponse {}
