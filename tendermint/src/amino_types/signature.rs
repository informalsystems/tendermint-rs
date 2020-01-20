use super::validate::ValidationError;
use crate::{chain, consensus};
use bytes::BufMut;
use prost_amino::{DecodeError, EncodeError};
use signatory::ed25519;

/// Amino messages which are signable within a Tendermint network
pub trait SignableMsg {
    /// Sign this message as bytes
    fn sign_bytes<B: BufMut>(
        &self,
        chain_id: chain::Id,
        sign_bytes: &mut B,
    ) -> Result<bool, EncodeError>;

    /// Set the Ed25519 signature on the underlying message
    fn set_signature(&mut self, sig: &ed25519::Signature);
    fn validate(&self) -> Result<(), ValidationError>;
    fn consensus_state(&self) -> Option<consensus::State>;
    fn height(&self) -> Option<i64>;
    fn msg_type(&self) -> Option<SignedMsgType>;
}

/// Signed message types. This follows:
/// <https://github.com/tendermint/tendermint/blob/455d34134cc53c334ebd3195ac22ea444c4b59bb/types/signed_msg_type.go#L3-L16>
#[derive(Copy, Clone, Debug)]
pub enum SignedMsgType {
    /// Votes
    PreVote,

    /// Commits
    PreCommit,

    /// Proposals
    Proposal,
}

impl SignedMsgType {
    pub fn to_u32(self) -> u32 {
        match self {
            // Votes
            Self::PreVote => 0x01,
            Self::PreCommit => 0x02,
            // Proposals
            Self::Proposal => 0x20,
        }
    }

    #[allow(dead_code)]
    fn from(data: u32) -> Result<Self, DecodeError> {
        match data {
            0x01 => Ok(Self::PreVote),
            0x02 => Ok(Self::PreCommit),
            0x20 => Ok(Self::Proposal),
            _ => Err(DecodeError::new("Invalid vote type")),
        }
    }
}
