use tendermint::{abci, evidence};
use tendermint_proto::v0_34::types::Evidence as RawEvidence;

use crate::prelude::*;
use crate::serializers::bytes::base64string;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone)]
pub struct Dialect;

impl crate::dialect::Dialect for Dialect {
    type Event = Event;
    type Evidence = Evidence;
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: Vec<EventAttribute>,
}

impl From<Event> for abci::Event {
    fn from(msg: Event) -> Self {
        Self {
            kind: msg.kind,
            attributes: msg.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<abci::Event> for Event {
    fn from(msg: abci::Event) -> Self {
        Self {
            kind: msg.kind,
            attributes: msg.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EventAttribute {
    /// The event key.
    #[serde(
        serialize_with = "base64string::serialize",
        deserialize_with = "base64string::deserialize_to_string"
    )]
    pub key: String,
    /// The event value.
    #[serde(
        serialize_with = "base64string::serialize",
        deserialize_with = "base64string::deserialize"
    )]
    pub value: Vec<u8>,
    /// Whether Tendermint's indexer should index this event.
    ///
    /// **This field is nondeterministic**.
    pub index: bool,
}

impl From<EventAttribute> for abci::EventAttribute {
    fn from(msg: EventAttribute) -> Self {
        Self::V034(abci::v0_34::EventAttribute {
            key: msg.key,
            value: msg.value,
            index: msg.index,
        })
    }
}

impl From<abci::EventAttribute> for EventAttribute {
    fn from(msg: abci::EventAttribute) -> Self {
        Self {
            key: msg.key().clone(),
            value: msg.value_as_bytes().to_vec(),
            index: msg.index(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(into = "RawEvidence", try_from = "RawEvidence")]
pub struct Evidence(evidence::Evidence);

impl From<Evidence> for RawEvidence {
    fn from(evidence: Evidence) -> Self {
        evidence.0.into()
    }
}

impl TryFrom<RawEvidence> for Evidence {
    type Error = <evidence::Evidence as TryFrom<RawEvidence>>::Error;

    fn try_from(value: RawEvidence) -> Result<Self, Self::Error> {
        Ok(Self(evidence::Evidence::try_from(value)?))
    }
}

impl From<evidence::Evidence> for Evidence {
    fn from(evidence: evidence::Evidence) -> Self {
        Self(evidence)
    }
}
