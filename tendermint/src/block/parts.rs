//! Block parts

use crate::hash::Algorithm;
use crate::hash::SHA256_HASH_SIZE;
use crate::Hash;
use crate::{Error, Kind};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tendermint_proto::types::{
    CanonicalPartSetHeader as RawCanonicalPartSetHeader, PartSetHeader as RawPartSetHeader,
};
use tendermint_proto::DomainType;

/// Block parts header
#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord,
)]
pub struct Header {
    /// Number of parts in this block
    pub total: u32,

    /// Hash of the parts set header,
    pub hash: Hash,
}

impl DomainType<RawPartSetHeader> for Header {}

impl TryFrom<RawPartSetHeader> for Header {
    type Error = Error;

    fn try_from(value: RawPartSetHeader) -> Result<Self, Self::Error> {
        if !value.hash.is_empty() && value.hash.len() != SHA256_HASH_SIZE {
            return Err(Kind::InvalidHashSize.into());
        }
        Ok(Self {
            total: value.total,
            hash: Hash::from_bytes(Algorithm::Sha256, &value.hash)?,
        })
    }
}

impl From<Header> for RawPartSetHeader {
    fn from(value: Header) -> Self {
        RawPartSetHeader {
            total: value.total,
            hash: value.hash.into(),
        }
    }
}

impl TryFrom<RawCanonicalPartSetHeader> for Header {
    type Error = Error;

    fn try_from(value: RawCanonicalPartSetHeader) -> Result<Self, Self::Error> {
        if !value.hash.is_empty() && value.hash.len() != SHA256_HASH_SIZE {
            return Err(Kind::InvalidHashSize.into());
        }
        Ok(Self {
            total: value.total,
            hash: Hash::from_bytes(Algorithm::Sha256, &value.hash)?,
        })
    }
}

impl From<Header> for RawCanonicalPartSetHeader {
    fn from(value: Header) -> Self {
        RawCanonicalPartSetHeader {
            total: value.total,
            hash: value.hash.into(),
        }
    }
}

impl Header {
    /// Create a new parts header
    pub fn new(total: u32, hash: Hash) -> Self {
        Header { total, hash }
    }
}
