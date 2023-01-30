use crate::prelude::*;
use crate::serializers::bytes::base64string;
use serde::{Deserialize, Serialize};

#[derive(Default)]
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
