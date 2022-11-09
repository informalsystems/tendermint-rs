//! Block parts

use serde::{Deserialize, Serialize};
use tendermint_proto::v0_37::types::PartSetHeader as RawPartSetHeader;

use crate::{error::Error, prelude::*, Hash};

/// Block parts header
#[derive(
    Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize,
)]
#[serde(try_from = "RawPartSetHeader", into = "RawPartSetHeader")] // Used by KMS state file
#[non_exhaustive]
pub struct Header {
    /// Number of parts in this block
    pub total: u32,

    /// Hash of the parts set header,
    pub hash: Hash,
}

tendermint_pb_modules! {
    use pb::types::{
        CanonicalPartSetHeader as RawCanonicalPartSetHeader, PartSetHeader as RawPartSetHeader,
    };
    use crate::{
        error::Error,
        hash::{Algorithm, SHA256_HASH_SIZE},
        prelude::*,
        Hash,
    };
    use super::Header;

    impl Protobuf<RawPartSetHeader> for Header {}

    impl TryFrom<RawPartSetHeader> for Header {
        type Error = Error;

        fn try_from(value: RawPartSetHeader) -> Result<Self, Self::Error> {
            if !value.hash.is_empty() && value.hash.len() != SHA256_HASH_SIZE {
                return Err(Error::invalid_hash_size());
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
                return Err(Error::invalid_hash_size());
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
}

impl Header {
    /// constructor
    pub fn new(total: u32, hash: Hash) -> Result<Self, Error> {
        if total == 0 && hash != Hash::None {
            return Err(Error::invalid_part_set_header(
                "zero total with existing hash".to_string(),
            ));
        }
        if total != 0 && hash == Hash::None {
            return Err(Error::invalid_part_set_header(
                "non-zero total with empty hash".to_string(),
            ));
        }
        Ok(Header { total, hash })
    }
}
