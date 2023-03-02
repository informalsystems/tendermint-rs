use crate::prelude::*;

#[doc = include_str!("../doc/response-echo.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Echo {
    /// The message sent in the request.
    pub message: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::Echo;

    impl From<Echo> for pb::abci::ResponseEcho {
        fn from(echo: Echo) -> Self {
            Self {
                message: echo.message,
            }
        }
    }

    impl TryFrom<pb::abci::ResponseEcho> for Echo {
        type Error = crate::Error;

        fn try_from(echo: pb::abci::ResponseEcho) -> Result<Self, Self::Error> {
            Ok(Self {
                message: echo.message,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseEcho> for Echo {}
}
