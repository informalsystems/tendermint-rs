//! SignedHeader contains commit and and block header.
//! It is what the rpc endpoint /commit returns and hence can be used by a
//! light client.
use serde::{Deserialize, Serialize};

use crate::block;
use crate::{Error, Kind};
use anomaly::format_err;
use std::convert::TryFrom;
use tendermint_proto::types::SignedHeader as RawSignedHeader;
use tendermint_proto::DomainType;

/// Signed block headers
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SignedHeader {
    /// Block header
    pub header: block::Header,
    /// Commit containing signatures for the header
    pub commit: block::Commit,
}

impl DomainType<RawSignedHeader> for SignedHeader {}

impl TryFrom<RawSignedHeader> for SignedHeader {
    type Error = Error;

    fn try_from(value: RawSignedHeader) -> Result<Self, Self::Error> {
        if value.header.is_none() {
            return Err(format_err!(Kind::InvalidSignedHeader, "empty header field").into());
        }
        if value.commit.is_none() {
            return Err(format_err!(Kind::InvalidSignedHeader, "empty commit field").into());
        }
        unimplemented!("Greg implement it using the fields' implementations")
    }
}

impl From<SignedHeader> for RawSignedHeader {
    fn from(value: SignedHeader) -> Self {
        RawSignedHeader {
            header: Some(value.header.into()),
            commit: Some(value.commit.into()),
        }
    }
}
