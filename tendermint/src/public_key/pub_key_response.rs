use crate::{privval::RemoteSignerError, PublicKey};

/// PubKeyResponse
#[derive(Clone, PartialEq, Eq, Debug)]
// Todo: either pub_key OR error is present
pub struct PubKeyResponse {
    /// Public key
    pub pub_key: Option<PublicKey>,

    /// Error
    pub error: Option<RemoteSignerError>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::PubKeyResponse;
    use pb::privval::PubKeyResponse as RawPubKeyResponse;

    impl Protobuf<RawPubKeyResponse> for PubKeyResponse {}

    impl TryFrom<RawPubKeyResponse> for PubKeyResponse {
        type Error = crate::Error;

        fn try_from(value: RawPubKeyResponse) -> Result<Self, Self::Error> {
            Ok(PubKeyResponse {
                pub_key: value.pub_key.map(TryInto::try_into).transpose()?,
                error: value.error.map(TryInto::try_into).transpose()?,
            })
        }
    }

    impl From<PubKeyResponse> for RawPubKeyResponse {
        fn from(value: PubKeyResponse) -> Self {
            RawPubKeyResponse {
                pub_key: value.pub_key.map(Into::into),
                error: value.error.map(Into::into),
            }
        }
    }
}
// Todo: write unit test
