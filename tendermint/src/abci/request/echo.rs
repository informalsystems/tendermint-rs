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

mod v0_34 {
    use tendermint_proto::v0_34::abci as pb;
    use tendermint_proto::Protobuf;

    use super::Echo;

    impl From<Echo> for pb::RequestEcho {
        fn from(echo: Echo) -> Self {
            Self {
                message: echo.message,
            }
        }
    }

    impl TryFrom<pb::RequestEcho> for Echo {
        type Error = crate::Error;

        fn try_from(echo: pb::RequestEcho) -> Result<Self, Self::Error> {
            Ok(Self {
                message: echo.message,
            })
        }
    }

    impl Protobuf<pb::RequestEcho> for Echo {}
}

mod v0_37 {
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    use super::Echo;

    impl From<Echo> for pb::RequestEcho {
        fn from(echo: Echo) -> Self {
            Self {
                message: echo.message,
            }
        }
    }

    impl TryFrom<pb::RequestEcho> for Echo {
        type Error = crate::Error;

        fn try_from(echo: pb::RequestEcho) -> Result<Self, Self::Error> {
            Ok(Self {
                message: echo.message,
            })
        }
    }

    impl Protobuf<pb::RequestEcho> for Echo {}
}
