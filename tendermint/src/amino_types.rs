//! Message types serialized using the Amino serialization format
//! <https://github.com/tendermint/amino_rs>

#![allow(missing_docs)]

pub mod block_id;
pub mod ed25519;
pub mod message;
pub mod proposal;
pub mod signature;
pub mod validate;
pub mod vote;

pub use self::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader, PartSetHeader},
    ed25519::{PubKeyRequest, PubKeyResponse},
    proposal::{SignProposalRequest, SignedProposalResponse},
    signature::{SignableMsg, SignedMsgType},
    validate::ConsensusMessage,
};
