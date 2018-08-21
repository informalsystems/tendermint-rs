use bytes::IntoBuf;
use prost::encoding::decode_varint;
use prost::Message;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::io::{self, Read};
use std::io::{Error, ErrorKind};
use types::{
    PoisonPillMsg, PubKeyMsg, SignHeartbeatMsg, SignProposalMsg, SignVoteMsg, HEARTBEAT_AMINO_NAME,
    POISON_PILL_AMINO_NAME, PROPOSAL_AMINO_NAME, PUBKEY_AMINO_NAME, VOTE_AMINO_NAME,
};

pub const MAX_MSG_LEN: usize = 1024;

/// Requests to the KMS
pub enum Request {
    /// Sign the given message
    SignHeartbeat(SignHeartbeatMsg),
    SignProposal(SignProposalMsg),
    SignVote(SignVoteMsg),
    ShowPublicKey(PubKeyMsg),

    /// Instruct the KMS to terminate
    PoisonPill(PoisonPillMsg),
}

use std::collections::HashMap;

fn compute_disfix(name: &str) -> (Vec<u8>) {
    let mut sh = Sha256::default();
    sh.input(name.as_bytes());
    let output = sh.result();

    let prefix_bytes: Vec<u8> = output
        .iter()
        .filter(|&x| *x != 0x00)
        .skip(3)
        .filter(|&x| *x != 0x00)
        .cloned()
        .take(4)
        .collect();

    prefix_bytes
}

// pre-compute registered types prefix (this is probably sth. our amino library should
// provide instead)
lazy_static! {
    static ref PP_PREFIX: Vec<u8> = compute_disfix(POISON_PILL_AMINO_NAME);
    static ref HEART_BEAT_PREFIX: Vec<u8> = compute_disfix(HEARTBEAT_AMINO_NAME);
    static ref VOTE_PREFIX: Vec<u8> = compute_disfix(VOTE_AMINO_NAME);
    static ref PROPOSAL_PREFIX: Vec<u8> = compute_disfix(PROPOSAL_AMINO_NAME);
    static ref PUBKEY_PREFIX: Vec<u8> = compute_disfix(PUBKEY_AMINO_NAME);
}

impl Request {
    /// Read a request from the given readable
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        // TODO(ismail):
        // 1) read length delimiter
        // 2) create buffer of this length
        // 3) read amino prefix
        // 4) depending on amino prefix call Message::decode_length_delimited on rest of
        // buffer

        // this buffer contains the overall length and the amino prefix (for the registered types)
        let mut buf = vec![0; MAX_MSG_LEN];

        r.read(&mut buf)?;

        let buff: &mut Cursor<Vec<u8>> = &mut buf.into_buf();
        let len = decode_varint(buff).unwrap();
        if len > MAX_MSG_LEN as u64 {
            return Err(Error::new(ErrorKind::InvalidData, "RPC message too large."));
        }
        // we read that many bytes:
        let mut amino_pre = vec![0; 4];
        buff.read_exact(&mut amino_pre);
        buff.set_position(0);
        // TODO: probably there is a way without cloning this:
        let rem: Vec<u8> = buff.clone().into_inner();
        if amino_pre == *PP_PREFIX {
            // do not spent any time decoding, we are going down anyways
            return Ok(Request::PoisonPill(PoisonPillMsg {}));
        } else if amino_pre == *HEART_BEAT_PREFIX {
            if let Ok(hb) = SignHeartbeatMsg::decode(&rem) {
                return Ok(Request::SignHeartbeat(hb));
            }
        } else if amino_pre == *VOTE_PREFIX {
            if let Ok(vote) = SignVoteMsg::decode(&rem) {
                return Ok(Request::SignVote(vote));
            }
        } else if amino_pre == *PROPOSAL_PREFIX {
            if let Ok(prop) = SignProposalMsg::decode(&rem) {
                return Ok(Request::SignProposal(prop));
            }
        } else if amino_pre == *PUBKEY_PREFIX {
            if let Ok(prop) = PubKeyMsg::decode_length_delimited(&rem) {
                return Ok(Request::ShowPublicKey(prop));
            }
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            "Received unknown RPC message.",
        ))
    }
}

/// Responses from the KMS
pub enum Response {
    /// Signature response
    SignedHeartBeat(SignHeartbeatMsg),
    SignedVote(SignVoteMsg),
    SignedProposal(SignProposalMsg),
    PublicKey(PubKeyMsg),
}
