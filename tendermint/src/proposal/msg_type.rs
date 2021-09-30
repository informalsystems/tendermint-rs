use crate::error::Error;
use crate::prelude::*;
use core::convert::TryFrom;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tendermint_proto::Protobuf;

/// Types of proposals
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    /// Regular Proposal
    Proposal = 32,
}

impl Protobuf<i32> for Type {}

impl TryFrom<i32> for Type {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            32 => Ok(Type::Proposal),
            _ => Err(Error::invalid_message_type()),
        }
    }
}

impl From<Type> for i32 {
    fn from(value: Type) -> Self {
        value as i32
    }
}

impl Serialize for Type {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        i32::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let byte = i32::deserialize(deserializer)?;
        Type::try_from(byte)
            .map_err(|_| D::Error::custom(format!("invalid proposal type: {}", byte)))
    }
}
