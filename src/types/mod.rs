extern crate prost;

mod ed25519msg;
mod heartbeat;
mod proposal;
mod vote;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq, Message)]
pub struct PartsSetHeader {
    #[prost(sint64, tag = "1")]
    total: i64,
    #[prost(bytes, tag = "2")]
    hash: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct BlockID {
    #[prost(bytes, tag = "1")]
    hash: Vec<u8>,
    #[prost(message, tag = "2")]
    parts_header: Option<PartsSetHeader>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Time {
    #[prost(sfixed64, tag = "1")]
    pub seconds: i64,
    #[prost(sfixed32, tag = "2")]
    pub nanos: i32,
}

/// Converts `Time` to a `SystemTime`.
impl From<Time> for SystemTime {
    fn from(time: Time) -> SystemTime {
        if time.seconds >= 0 {
            UNIX_EPOCH + Duration::new(time.seconds as u64, time.nanos as u32)
        } else {
            UNIX_EPOCH - Duration::new(time.seconds as u64, time.nanos as u32)
        }
    }
}

pub trait TendermintSign {
    fn cannonicalize(self, chain_id: &str) -> String;
    fn sign(&mut self);
}

pub use self::heartbeat::SignHeartbeatMsg;
pub use self::proposal::SignProposalMsg;
pub use self::vote::SignVoteMsg;
