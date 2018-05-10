use signatory::ed25519::Signature;

mod heartbeat;
mod proposal;
mod vote;

pub type ValidatorAddress = [u8; 20];

#[derive(PartialEq, Debug)]
pub struct PartsSetHeader {
    total: i64,
    hash: Vec<u8>,
}

#[derive(PartialEq, Debug)]
struct BlockID {
    hash: Vec<u8>,
    parts_header: PartsSetHeader,
}

pub trait TendermintSign {
    fn cannonicalize(self, chain_id: &str) -> String;
}

pub use self::heartbeat::Heartbeat;
