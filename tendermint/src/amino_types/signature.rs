use super::validate;
use crate::{chain, consensus};
use bytes::BufMut;
use prost::{DecodeError, EncodeError};
use tendermint_proto::types::SignedMsgType as RawSignedMsgType;

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
    fn validate(&self) -> Result<(), validate::Error>;
    fn consensus_state(&self) -> Option<consensus::State>;
    fn height(&self) -> Option<i64>;
    fn msg_type(&self) -> Option<SignedMsgType>;
}

/// SignedMsgType is a type of signed message in the consensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignedMsgType {
    Unknown = 0,
    /// Votes
    Prevote = 1,
    Precommit = 2,
    /// Proposals
    Proposal = 32,
}

impl From<RawSignedMsgType> for SignedMsgType {
    fn from(value: RawSignedMsgType) -> Self {
        match value {
            RawSignedMsgType::Unknown => SignedMsgType::Unknown,
            RawSignedMsgType::Prevote => SignedMsgType::Prevote,
            RawSignedMsgType::Precommit => SignedMsgType::Precommit,
            RawSignedMsgType::Proposal => SignedMsgType::Proposal,
        }
    }
}

impl From<SignedMsgType> for RawSignedMsgType {
    fn from(value: SignedMsgType) -> Self {
        match value {
            SignedMsgType::Unknown => RawSignedMsgType::Unknown,
            SignedMsgType::Prevote => RawSignedMsgType::Prevote,
            SignedMsgType::Precommit => RawSignedMsgType::Precommit,
            SignedMsgType::Proposal => RawSignedMsgType::Proposal,
        }
    }
}

impl PartialEq<SignedMsgType> for i32 {
    fn eq(&self, other: &SignedMsgType) -> bool {
        *self == *other as i32
    }
}

impl PartialEq<SignedMsgType> for u32 {
    fn eq(&self, other: &SignedMsgType) -> bool {
        *self == *other as u32
    }
}

impl SignedMsgType {
    #[allow(dead_code)]
    fn from(data: u32) -> Result<SignedMsgType, DecodeError> {
        match data {
            0x00 => Ok(SignedMsgType::Unknown),
            0x01 => Ok(SignedMsgType::Prevote),
            0x02 => Ok(SignedMsgType::Precommit),
            0x20 => Ok(SignedMsgType::Proposal),
            _ => Err(DecodeError::new("Invalid vote type")),
        }
    }
}
