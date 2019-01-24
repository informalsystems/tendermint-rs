//! Message types serialized using the Amino serialization format
//! <https://github.com/tendermint/amino_rs>

#![allow(missing_docs)]

pub mod block_id;
pub mod ed25519;
pub mod ping;
pub mod proposal;
pub mod remote_error;
pub mod secret_connection;
pub mod signature;
pub mod time;
pub mod validate;
pub mod vote;

pub use self::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader, PartsSetHeader},
    ed25519::{PubKeyRequest, PubKeyResponse, AMINO_NAME as PUBKEY_AMINO_NAME},
    ping::{PingRequest, PingResponse, AMINO_NAME as PING_AMINO_NAME},
    proposal::{SignProposalRequest, SignedProposalResponse, AMINO_NAME as PROPOSAL_AMINO_NAME},
    remote_error::RemoteError,
    secret_connection::AuthSigMessage,
    signature::{SignableMsg, SignedMsgType},
    time::TimeMsg,
    validate::ConsensusMessage,
    vote::{SignVoteRequest, SignedVoteResponse, AMINO_NAME as VOTE_AMINO_NAME},
};
