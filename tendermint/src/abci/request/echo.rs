use crate::prelude::*;

#[doc = include_str!("../doc/request-echo.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Echo {
    /// The message to send back.
    pub message: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::Echo;

    impl From<Echo> for pb::abci::RequestEcho {
        fn from(echo: Echo) -> Self {
            Self {
                message: echo.message,
            }
        }
    }

    impl TryFrom<pb::abci::RequestEcho> for Echo {
        type Error = crate::Error;

        fn try_from(echo: pb::abci::RequestEcho) -> Result<Self, Self::Error> {
            Ok(Self {
                message: echo.message,
            })
        }
    }

    impl Protobuf<pb::abci::RequestEcho> for Echo {}
}
