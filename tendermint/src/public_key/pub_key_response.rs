use crate::{Error, PublicKey};
use core::convert::{TryFrom, TryInto};
use tendermint_proto::privval::{PubKeyResponse as RawPubKeyResponse, RemoteSignerError};
use tendermint_proto::Protobuf;

/// PubKeyResponse
#[derive(Clone, PartialEq, Debug)]
// Todo: either pub_key OR error is present
pub struct PubKeyResponse {
    /// Public key
    pub pub_key: Option<PublicKey>,

    /// Error
    pub error: Option<RemoteSignerError>,
}

impl Protobuf<RawPubKeyResponse> for PubKeyResponse {}

impl TryFrom<RawPubKeyResponse> for PubKeyResponse {
    type Error = Error;

    fn try_from(value: RawPubKeyResponse) -> Result<Self, Self::Error> {
        Ok(PubKeyResponse {
            pub_key: value.pub_key.map(TryInto::try_into).transpose()?,
            error: value.error,
        })
    }
}

impl From<PubKeyResponse> for RawPubKeyResponse {
    fn from(value: PubKeyResponse) -> Self {
        RawPubKeyResponse {
            pub_key: value.pub_key.map(Into::into),
            error: value.error,
        }
    }
}

// Todo: write unit test
