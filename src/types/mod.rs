extern crate prost_amino;

pub mod ed25519msg;
pub mod heartbeat;
pub mod poisonpill;
pub mod proposal;
pub mod vote;

use bytes::BufMut;
use prost::encoding::{decode_varint, encoded_len_varint};
use prost::DecodeError;
use signatory::{Ed25519Signature, Signature};
use std::io::Cursor;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq, Message)]
pub struct PartsSetHeader {
    #[prost(sint64, tag = "1")]
    pub total: i64,
    #[prost(bytes, tag = "2")]
    pub hash: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalPartSetHeader {
    #[prost(bytes, tag = "1")]
    hash: Vec<u8>,
    #[prost(sint64, tag = "2")]
    total: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct BlockID {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(message, tag = "2")]
    pub parts_header: Option<PartsSetHeader>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalBlockID {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(message, tag = "2")]
    pub parts_header: Option<CanonicalPartSetHeader>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Time {
    // TODO(ismail): switch to protobuf's well known type as soon as
    // https://github.com/tendermint/go-amino/pull/224 was merged
    // and tendermint caught up on the latest amino release.
    #[prost(sfixed64, tag = "1")]
    pub seconds: i64,
    #[prost(sfixed32, tag = "2")]
    pub nanos: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteError {
    #[prost(sint32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub description: String,
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

pub trait TendermintSignable {
    fn sign_bytes<B>(
        &self,
        chain_id: &str,
        sign_bytes: &mut B,
    ) -> Result<bool, prost_amino::EncodeError>
    where
        B: BufMut;
    fn set_signature(&mut self, sig: &Ed25519Signature);
}

// This follows:
// https://github.com/tendermint/tendermint/blob/455d34134cc53c334ebd3195ac22ea444c4b59bb/types/signed_msg_type.go#L3-L16
#[derive(Debug, Copy)]
enum SignedMsgType {
    // Votes
    PreVote,
    PreCommit,

    // Proposals
    Proposal,

    // Heartbeat
    Heartbeat,
}

pub fn extract_actual_len(buf: &[u8]) -> Result<u64, DecodeError> {
    let mut buff = Cursor::new(buf);
    let actual_len = decode_varint(&mut buff)?;
    if actual_len == 0 {
        return Ok(1);
    }
    Ok(actual_len + (encoded_len_varint(actual_len) as u64))
}

impl SignedMsgType {
    fn to_u32(self) -> u32 {
        match self {
            // Votes
            SignedMsgType::PreVote => 0x01,
            SignedMsgType::PreCommit => 0x02,
            // Proposals
            SignedMsgType::Proposal => 0x20,
            // Heartbeat
            SignedMsgType::Heartbeat => 0x30,
        }
    }

    fn from(data: u32) -> Result<SignedMsgType, DecodeError> {
        match data {
            0x01 => Ok(SignedMsgType::PreVote),
            0x02 => Ok(SignedMsgType::PreCommit),
            0x20 => Ok(SignedMsgType::Proposal),
            0x30 => Ok(SignedMsgType::Heartbeat),
            _ => Err(DecodeError::new("Invalid vote type")),
        }
    }

    fn is_valid_vote_type(msg_type: SignedMsgType) -> bool {
        match msg_type {
            SignedMsgType::PreVote => true,
            SignedMsgType::PreCommit => true,
            _ => false,
        }
    }
}

impl Clone for SignedMsgType {
    fn clone(&self) -> SignedMsgType {
        *self
    }
}

pub use self::ed25519msg::PubKeyMsg;
pub use self::ed25519msg::AMINO_NAME as PUBKEY_AMINO_NAME;
pub use self::heartbeat::SignHeartbeatRequest;
pub use self::heartbeat::SignedHeartbeatResponse;
pub use self::heartbeat::AMINO_NAME as HEARTBEAT_AMINO_NAME;
pub use self::poisonpill::PoisonPillMsg;
pub use self::poisonpill::AMINO_NAME as POISON_PILL_AMINO_NAME;
pub use self::proposal::SignProposalRequest;
pub use self::proposal::SignedProposalResponse;
pub use self::proposal::AMINO_NAME as PROPOSAL_AMINO_NAME;
pub use self::vote::SignVoteRequest;
pub use self::vote::SignedVoteResponse;
pub use self::vote::AMINO_NAME as VOTE_AMINO_NAME;
