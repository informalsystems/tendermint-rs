//! RPC subscription event-related data structures.

use alloc::collections::BTreeMap as HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    client::subscription::SubscriptionEvent, event::EventData, prelude::*, query::EventType,
    response::Wrapper, Response,
};

/// An incoming event produced by a [`Subscription`].
///
/// In Tendermint 0.34, the format of the `events` field was a key-value map,
/// with key names encoding both the event type and a tag.
///
/// [`Subscription`]: ../struct.Subscription.html
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    /// The query that produced the event.
    pub query: String,
    /// The data associated with the event.
    pub data: EventData,
    /// Event type and attributes map.
    pub events: Option<HashMap<String, Vec<String>>>,
}
impl Response for Event {}

impl SubscriptionEvent for Event {
    fn query(&self) -> &str {
        &self.query
    }
}

/// A JSON-RPC-wrapped event.
pub type WrappedEvent = Wrapper<Event>;

impl Event {
    /// Returns the type associated with this event, if we recognize it.
    ///
    /// Returns `None` if we don't yet support this event type.
    pub fn event_type(&self) -> Option<EventType> {
        match self.data {
            EventData::NewBlock { .. } => Some(EventType::NewBlock),
            EventData::Tx { .. } => Some(EventType::Tx),
            _ => None,
        }
    }
}
