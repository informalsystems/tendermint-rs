use crate::prelude::*;

/// An event that occurred while processing a request.
///
/// Application developers can attach additional information to
/// [`BeginBlock`](super::response::BeginBlock),
/// [`EndBlock`](super::response::EndBlock),
/// [`CheckTx`](super::response::CheckTx), and
/// [`DeliverTx`](super::response::DeliverTx) responses. Later, transactions may
/// be queried using these events.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#events)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Event {
    /// The kind of event.
    ///
    /// Tendermint calls this the `type`, but we use `kind` to avoid confusion
    /// with Rust types and follow Rust conventions.
    pub kind: String,
    /// A list of [`EventAttribute`]s describing the event.
    pub attributes: Vec<EventAttribute>,
}

impl Event {
    /// Construct an event from generic data.
    ///
    /// The `From` impls on [`EventAttribute`] and the [`EventAttributeIndexExt`]
    /// trait allow ergonomic event construction, as in this example:
    ///
    /// ```
    /// use tendermint::abci::{Event, EventAttributeIndexExt};
    ///
    /// let event = Event::new(
    ///     "app",
    ///     vec![
    ///         ("key1", "value1").index(),
    ///         ("key2", "value2").index(),
    ///         ("key3", "value3").no_index(), // will not be indexed
    ///     ],
    /// );
    /// ```
    // XXX(hdevalence): remove vec! from example after https://github.com/rust-lang/rust/pull/65819
    pub fn new<K, I>(kind: K, attributes: I) -> Self
    where
        K: Into<String>,
        I: IntoIterator,
        I::Item: Into<EventAttribute>,
    {
        Self {
            kind: kind.into(),
            attributes: attributes.into_iter().map(Into::into).collect(),
        }
    }
}

/// A key-value pair describing an [`Event`].
///
/// Generic methods are provided for more ergonomic attribute construction, see
/// [`Event::new`] for details.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#events)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EventAttribute {
    /// The event key.
    pub key: String,
    /// The event value.
    pub value: String,
    /// Whether Tendermint's indexer should index this event.
    ///
    /// **This field is nondeterministic**.
    pub index: bool,
}

impl<K: Into<String>, V: Into<String>> From<(K, V, bool)> for EventAttribute {
    fn from((key, value, index): (K, V, bool)) -> Self {
        EventAttribute {
            key: key.into(),
            value: value.into(),
            index,
        }
    }
}

/// Adds convenience methods to tuples for more ergonomic [`EventAttribute`]
/// construction.
///
/// See [`Event::new`] for details.
#[allow(missing_docs)]
pub trait EventAttributeIndexExt: private::Sealed {
    type Key;
    type Value;

    /// Indicate that this key/value pair should be indexed by Tendermint.
    fn index(self) -> (Self::Key, Self::Value, bool);
    /// Indicate that this key/value pair should not be indexed by Tendermint.
    fn no_index(self) -> (Self::Key, Self::Value, bool);
}

impl<K: Into<String>, V: Into<String>> EventAttributeIndexExt for (K, V) {
    type Key = K;
    type Value = V;
    fn index(self) -> (K, V, bool) {
        let (key, value) = self;
        (key, value, true)
    }
    fn no_index(self) -> (K, V, bool) {
        let (key, value) = self;
        (key, value, false)
    }
}

mod private {
    use crate::prelude::*;

    pub trait Sealed {}

    impl<K: Into<String>, V: Into<String>> Sealed for (K, V) {}
}

impl<K: Into<String>, V: Into<String>> From<(K, V)> for EventAttribute {
    fn from((key, value): (K, V)) -> Self {
        (key, value, false).into()
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};

use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<EventAttribute> for pb::EventAttribute {
    fn from(event: EventAttribute) -> Self {
        Self {
            key: event.key,
            value: event.value,
            index: event.index,
        }
    }
}

impl TryFrom<pb::EventAttribute> for EventAttribute {
    type Error = crate::Error;

    fn try_from(event: pb::EventAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            key: event.key,
            value: event.value,
            index: event.index,
        })
    }
}

impl Protobuf<pb::EventAttribute> for EventAttribute {}

impl From<Event> for pb::Event {
    fn from(event: Event) -> Self {
        Self {
            r#type: event.kind,
            attributes: event.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::Event> for Event {
    type Error = crate::Error;

    fn try_from(event: pb::Event) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: event.r#type,
            attributes: event
                .attributes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<pb::Event> for Event {}
