use tendermint::abci;

use crate::prelude::*;
use crate::serializers::bytes::base64string;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone)]
pub struct Dialect;

impl crate::dialect::Dialect for Dialect {
    type Event = Event;
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
        deserialize_with = "base64string::deserialize_to_string"
    )]
    pub value: String,
    /// Whether Tendermint's indexer should index this event.
    ///
    /// **This field is nondeterministic**.
    pub index: bool,
}

impl From<EventAttribute> for abci::EventAttribute {
    fn from(msg: EventAttribute) -> Self {
        Self {
            key: msg.key,
            value: msg.value,
            index: msg.index,
        }
    }
}
