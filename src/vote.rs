//! Votes from validators

mod power;

pub use self::power::Power;
use crate::{account, amino_types, block, Signature, Time};
use signatory::Signature as SignatureTrait;
use {
    crate::serializers,
    serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer},
};

/// Votes are signed messages from validators for a particular block which
/// include information about the validator signing it.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#vote>
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vote {
    /// Type of vote (prevote or precommit)
    #[serde(rename = "type")]
    pub vote_type: Type,

    /// Block height
    pub height: block::Height,

    /// Round
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub round: u64,

    /// Block ID
    pub block_id: block::Id,

    /// Timestamp
    pub timestamp: Time,

    /// Validator address
    pub validator_address: account::Id,

    /// Validator index
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub validator_index: u64,

    /// Signature
    pub signature: Signature,
}

impl Vote {
    /// Convert to amino_types vote
    pub fn to_amino(&self) -> amino_types::vote::Vote {
        amino_types::vote::Vote {
            vote_type: self.vote_type.to_u8() as u32,
            height: self.height.value() as i64, // XXX: careful integer overflow
            round: self.round as i64,           // XXX: careful integer overflow
            block_id: Some(self.block_id.to_amino()), // TODO: fill in this to_amino method
            timestamp: Some(amino_types::TimeMsg::from(self.timestamp)),
            validator_address: self.validator_address.as_bytes().to_vec(),
            validator_index: self.validator_index as i64,
            signature: self.signature.ed25519().unwrap().as_slice().to_vec(), // TODO: support more than ed25519
        }
    }
}

impl Vote {
    /// Is this vote a prevote?
    pub fn is_prevote(&self) -> bool {
        match self.vote_type {
            Type::Prevote => true,
            Type::Precommit => false,
        }
    }

    /// Is this vote a precommit?
    pub fn is_precommit(&self) -> bool {
        match self.vote_type {
            Type::Precommit => true,
            Type::Prevote => false,
        }
    }
}

/// Types of votes
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    /// Votes for blocks which validators observe are valid for a given round
    Prevote = 1,

    /// Votes to commit to a particular block for a given round
    Precommit = 2,
}

impl Type {
    /// Deserialize this type from a byte
    pub fn from_u8(byte: u8) -> Option<Type> {
        match byte {
            1 => Some(Type::Prevote),
            2 => Some(Type::Precommit),
            _ => None,
        }
    }

    /// Serialize this type as a byte
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl Serialize for Type {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_u8().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let byte = u8::deserialize(deserializer)?;
        Type::from_u8(byte).ok_or_else(|| D::Error::custom(format!("invalid vote type: {}", byte)))
    }
}
