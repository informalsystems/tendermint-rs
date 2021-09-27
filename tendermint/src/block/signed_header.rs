//! SignedHeader contains commit and and block header.
//! It is what the rpc endpoint /commit returns and hence can be used by a
//! light client.
use crate::{block, Error};
use core::convert::{TryFrom, TryInto};
use serde::{Deserialize, Serialize, Serializer};
use tendermint_proto::{types::SignedHeader as RawSignedHeader, Protobuf};

/// Signed block headers
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawSignedHeader")] // used by RPC /commit endpoint
#[non_exhaustive]
pub struct SignedHeader {
    /// Block header
    pub header: block::Header,
    /// Commit containing signatures for the header
    pub commit: block::Commit,
}

impl TryFrom<RawSignedHeader> for SignedHeader {
    type Error = Error;

    fn try_from(value: RawSignedHeader) -> Result<Self, Self::Error> {
        let header = value
            .header
            .ok_or_else(Error::invalid_signed_header)?
            .try_into()?;
        let commit = value
            .commit
            .ok_or_else(Error::invalid_signed_header)?
            .try_into()?;
        Self::new(header, commit) // Additional checks
    }
}

impl TryFrom<SignedHeader> for RawSignedHeader {
    type Error = Error;

    fn try_from(value: SignedHeader) -> Result<Self, Error> {
        Ok(RawSignedHeader {
            header: Some(value.header.try_into()?),
            commit: Some(value.commit.try_into()?),
        })
    }
}

impl Serialize for SignedHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw: RawSignedHeader = self.clone().try_into().map_err(serde::ser::Error::custom)?;

        raw.serialize(serializer)
    }
}

impl Protobuf<RawSignedHeader> for SignedHeader {}

impl SignedHeader {
    /// Constructor.
    pub fn new(header: block::Header, commit: block::Commit) -> Result<Self, Error> {
        if header.height != commit.height {
            return Err(Error::invalid_signed_header());
        }
        Ok(Self { header, commit })
    }

    /// Get header
    pub fn header(&self) -> &block::Header {
        &self.header
    }

    /// Get commit
    pub fn commit(&self) -> &block::Commit {
        &self.commit
    }
}
