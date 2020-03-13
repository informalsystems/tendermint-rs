//! Message types serialized using the Amino serialization format
//! <https://github.com/tendermint/amino_rs>

#![allow(missing_docs)]

pub mod block_id;
pub mod ed25519;
pub mod message;
pub mod ping;
pub mod proposal;
pub mod remote_error;
pub mod signature;
pub mod time;
pub mod validate;
pub mod version;
pub mod vote;

pub use self::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader, PartsSetHeader},
    ed25519::{
        PubKeyRequest, PubKeyResponse, AMINO_NAME as PUBKEY_AMINO_NAME,
        AMINO_PREFIX as PUBKEY_PREFIX,
    },
    ping::{PingRequest, PingResponse, AMINO_NAME as PING_AMINO_NAME, AMINO_PREFIX as PING_PREFIX},
    proposal::{
        SignProposalRequest, SignedProposalResponse, AMINO_NAME as PROPOSAL_AMINO_NAME,
        AMINO_PREFIX as PROPOSAL_PREFIX,
    },
    remote_error::RemoteError,
    signature::{SignableMsg, SignedMsgType},
    time::TimeMsg,
    validate::ConsensusMessage,
    version::ConsensusVersion,
    vote::{
        SignVoteRequest, SignedVoteResponse, AMINO_NAME as VOTE_AMINO_NAME,
        AMINO_PREFIX as VOTE_PREFIX,
    },
};

use sha2::{Digest, Sha256};

/// Compute the Amino prefix for the given registered type name
pub fn compute_prefix(name: &str) -> Vec<u8> {
    let mut sh = Sha256::default();
    sh.input(name.as_bytes());
    let output = sh.result();

    output
        .iter()
        .filter(|&x| *x != 0x00)
        .skip(3)
        .filter(|&x| *x != 0x00)
        .cloned()
        .take(4)
        .collect()
}
