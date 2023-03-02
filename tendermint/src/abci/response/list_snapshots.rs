use super::super::types::Snapshot;
use crate::prelude::*;

#[doc = include_str!("../doc/response-listsnapshots.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ListSnapshots {
    /// A list of local state snapshots.
    pub snapshots: Vec<Snapshot>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::ListSnapshots;

    impl From<ListSnapshots> for pb::abci::ResponseListSnapshots {
        fn from(list_snapshots: ListSnapshots) -> Self {
            Self {
                snapshots: list_snapshots
                    .snapshots
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseListSnapshots> for ListSnapshots {
        type Error = crate::Error;

        fn try_from(list_snapshots: pb::abci::ResponseListSnapshots) -> Result<Self, Self::Error> {
            Ok(Self {
                snapshots: list_snapshots
                    .snapshots
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseListSnapshots> for ListSnapshots {}
}
