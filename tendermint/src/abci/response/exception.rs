use crate::prelude::*;

#[doc = include_str!("../doc/response-exception.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Exception {
    /// Undocumented.
    pub error: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::Exception;

    impl From<Exception> for pb::abci::ResponseException {
        fn from(exception: Exception) -> Self {
            Self {
                error: exception.error,
            }
        }
    }

    impl TryFrom<pb::abci::ResponseException> for Exception {
        type Error = crate::Error;

        fn try_from(exception: pb::abci::ResponseException) -> Result<Self, Self::Error> {
            Ok(Self {
                error: exception.error,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseException> for Exception {}
}
