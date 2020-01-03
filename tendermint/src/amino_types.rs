//! Message types serialized using the Amino serialization format
//! <https://github.com/tendermint/amino_rs>

#![allow(missing_docs)]

mod block_id;
mod ed25519;
mod message;
mod ping;
mod proposal;
mod remote_error;
mod signature;
mod time;
mod validate;
mod version;
mod vote;

pub use self::{
    block_id::{BlockId, Canonical as CanonicalBlockId, CanonicalPartSetHeader, PartsSetHeader},
    ed25519::{PubKeyRequest, PubKeyResponse, AMINO_NAME as PUBKEY_AMINO_NAME},
    message::Message as AminoMessage,
    ping::{Request as PingRequest, Response as PingResponse, AMINO_NAME as PING_AMINO_NAME},
    proposal::{SignProposalRequest, SignedProposalResponse, AMINO_NAME as PROPOSAL_AMINO_NAME},
    remote_error::RemoteError,
    signature::{SignableMsg, SignedMsgType},
    time::TimeMsg,
    validate::ConsensusMessage,
    version::Consensus as ConsensusVersion,
    vote::{
        CanonicalVote, SignVoteRequest, SignedVoteResponse, Vote, AMINO_NAME as VOTE_AMINO_NAME,
    },
};
