use crate::prelude::*;

#[doc = include_str!("../doc/request-info.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Info {
    /// The Tendermint software semantic version.
    pub version: String,
    /// The Tendermint block protocol version.
    pub block_version: u64,
    /// The Tendermint p2p protocol version.
    pub p2p_version: u64,
    /// The ABCI protocol version.
    pub abci_version: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_34 {
    use super::Info;
    use tendermint_proto::v0_34::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<Info> for pb::RequestInfo {
        fn from(info: Info) -> Self {
            Self {
                version: info.version,
                block_version: info.block_version,
                p2p_version: info.p2p_version,
            }
        }
    }

    impl TryFrom<pb::RequestInfo> for Info {
        type Error = crate::Error;

        fn try_from(info: pb::RequestInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                version: info.version,
                block_version: info.block_version,
                p2p_version: info.p2p_version,
                abci_version: Default::default(),
            })
        }
    }

    impl Protobuf<pb::RequestInfo> for Info {}
}

mod v0_37 {
    use super::Info;
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<Info> for pb::RequestInfo {
        fn from(info: Info) -> Self {
            Self {
                version: info.version,
                block_version: info.block_version,
                p2p_version: info.p2p_version,
                abci_version: info.abci_version,
            }
        }
    }

    impl TryFrom<pb::RequestInfo> for Info {
        type Error = crate::Error;

        fn try_from(info: pb::RequestInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                version: info.version,
                block_version: info.block_version,
                p2p_version: info.p2p_version,
                abci_version: info.abci_version,
            })
        }
    }

    impl Protobuf<pb::RequestInfo> for Info {}
}

mod v0_38 {
    use super::Info;
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<Info> for pb::RequestInfo {
        fn from(info: Info) -> Self {
            Self {
                version: info.version,
                block_version: info.block_version,
                p2p_version: info.p2p_version,
                abci_version: info.abci_version,
            }
        }
    }

    impl TryFrom<pb::RequestInfo> for Info {
        type Error = crate::Error;

        fn try_from(info: pb::RequestInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                version: info.version,
                block_version: info.block_version,
                p2p_version: info.p2p_version,
                abci_version: info.abci_version,
            })
        }
    }

    impl Protobuf<pb::RequestInfo> for Info {}
}
