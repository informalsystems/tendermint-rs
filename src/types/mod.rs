extern crate prost_amino;

pub mod ed25519msg;
pub mod heartbeat;
pub mod poisonpill;
pub mod proposal;
pub mod vote;

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
    // TODO(ismail): can't the signing op time out or error in another way
    // (e.g.hsm module not found)
    // also, if we want to keep this method, we need the signer / priv key to be known here
    // probably the cannonicalize method is sufficient and the actual signing happens
    // outside of the type:
    fn sign(&mut self);
}

pub use self::ed25519msg::PubKeyMsg;
pub use self::ed25519msg::AMINO_NAME as PUBKEY_AMINO_NAME;
pub use self::heartbeat::SignHeartbeatMsg;
pub use self::heartbeat::AMINO_NAME as HEARTBEAT_AMINO_NAME;
pub use self::poisonpill::PoisonPillMsg;
pub use self::poisonpill::AMINO_NAME as POISON_PILL_AMINO_NAME;
pub use self::proposal::SignProposalMsg;
pub use self::proposal::AMINO_NAME as PROPOSAL_AMINO_NAME;
pub use self::vote::SignVoteMsg;
pub use self::vote::AMINO_NAME as VOTE_AMINO_NAME;
