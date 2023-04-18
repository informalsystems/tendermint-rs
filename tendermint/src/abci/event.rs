use serde::Serialize;

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
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Hash)]
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
    ///     [
    ///         ("key1", "value1").index(),
    ///         ("key2", "value2").index(),
    ///         ("key3", "value3").no_index(), // will not be indexed
    ///     ],
    /// );
    /// ```
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

    /// Checks whether `&self` is equal to `other`, ignoring the `index` field on any attributes.
    pub fn eq_ignoring_index(&self, other: &Self) -> bool {
        self.kind == other.kind
            // IMPORTANT! We need to check the lengths before calling zip,
            // in order to not drop any attributes.
            && self.attributes.len() == other.attributes.len()
            && self
                .attributes
                .iter()
                .zip(other.attributes.iter())
                .all(|(a, b)| a.eq_ignoring_index(b))
    }

    /// A variant of [`core::hash::Hash::hash`] that ignores the `index` field on any attributes.
    pub fn hash_ignoring_index<H: core::hash::Hasher>(&self, state: &mut H) {
        use core::hash::Hash;
        self.kind.hash(state);
        // We can't forward to the slice impl here, because we need to call `hash_ignoring_index`
        // on each attribute, so we need to do our own length prefixing.
        state.write_usize(self.attributes.len());
        for attr in &self.attributes {
            attr.hash_ignoring_index(state);
        }
    }
}

/// A marker trait for types that can be converted to and from [`Event`]s.
///
/// This trait doesn't make any assumptions about how the conversion is
/// performed, or how the type's data is encoded in event attributes.  Instead,
/// it just declares the conversion methods used to serialize the type to an
/// [`Event`] and to deserialize it from an [`Event`], allowing downstream users
/// to declare a single source of truth about how event data is structured.
///
/// # Contract
///
/// If `T: TypedEvent`, then:
///
/// - `T::try_from(e) == Ok(t)` for all `t: T, e: Event` where `Event::from(t).eq_ignoring_index(e)
///   == true`.
/// - `Event::from(T::try_from(e).unwrap()).eq_ignoring_index(e) == true` for all `e: Event` where
///   `T::try_from(e)` returns `Ok(_)`.
///
/// In other words, the conversion methods should round-trip on the attributes,
/// but are not required to preserve the (nondeterministic) index information.
pub trait TypedEvent
where
    Self: TryFrom<Event>,
    Event: From<Self>,
{
    /// Convenience wrapper around `Into::into` that doesn't require type inference.
    fn into_event(self) -> Event {
        self.into()
    }
}

/// A key-value pair describing an [`Event`].
///
/// Generic methods are provided for more ergonomic attribute construction, see
/// [`Event::new`] for details.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#events)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Hash)]
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

impl EventAttribute {
    /// Checks whether `&self` is equal to `other`, ignoring the `index` field.
    pub fn eq_ignoring_index(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }

    /// A variant of [`core::hash::Hash::hash`] that ignores the `index` field.
    pub fn hash_ignoring_index<H: core::hash::Hasher>(&self, state: &mut H) {
        use core::hash::Hash;
        // Call the `Hash` impl on the (k,v) tuple to avoid prefix collision issues.
        (&self.key, &self.value).hash(state);
    }
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

mod v0_34 {
    use super::{Event, EventAttribute};
    use crate::prelude::*;
    use core::convert::{TryFrom, TryInto};

    use tendermint_proto::v0_34::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<EventAttribute> for pb::EventAttribute {
        fn from(event: EventAttribute) -> Self {
            Self {
                key: event.key.into(),
                value: event.value.into(),
                index: event.index,
            }
        }
    }

    impl TryFrom<pb::EventAttribute> for EventAttribute {
        type Error = crate::Error;

        fn try_from(event: pb::EventAttribute) -> Result<Self, Self::Error> {
            // We insist that keys and values are strings, like tm 0.35 did.
            Ok(Self {
                key: String::from_utf8(event.key.to_vec())
                    .map_err(|e| crate::Error::parse(e.to_string()))?,
                value: String::from_utf8(event.value.to_vec())
                    .map_err(|e| crate::Error::parse(e.to_string()))?,
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
}

mod v0_37 {
    use super::{Event, EventAttribute};
    use crate::prelude::*;
    use core::convert::{TryFrom, TryInto};

    use tendermint_proto::v0_37::abci as pb;
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
            // We insist that keys and values are strings, like tm 0.35 did.
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::redundant_clone)]

    use serde::Deserialize;

    use super::*;

    #[test]
    fn event_eq_ignoring_index_ignores_index() {
        let event_a = Event::new("test", [("foo", "bar").index()]);
        let event_b = Event::new("test", [("foo", "bar").no_index()]);
        let event_c = Event::new("test", [("foo", "baz").index()]);

        assert_eq!(event_a.eq_ignoring_index(&event_b), true);
        assert_eq!(event_a.eq_ignoring_index(&event_c), false);
    }

    #[test]
    fn exercise_typed_event() {
        #[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Payload {
            x: u32,
            y: u32,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct MyEvent {
            a: Payload,
            b: Payload,
        }

        impl From<MyEvent> for Event {
            fn from(event: MyEvent) -> Self {
                Event::new(
                    "my_event",
                    vec![
                        ("a", serde_json::to_string(&event.a).unwrap()).index(),
                        ("b", serde_json::to_string(&event.b).unwrap()).index(),
                    ],
                )
            }
        }

        impl TryFrom<Event> for MyEvent {
            type Error = (); // Avoid depending on a specific error library in test code

            fn try_from(event: Event) -> Result<Self, Self::Error> {
                if event.kind != "my_event" {
                    return Err(());
                }

                let a = event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "a")
                    .ok_or(())
                    .and_then(|attr| serde_json::from_str(&attr.value).map_err(|_| ()))?;
                let b = event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "b")
                    .ok_or(())
                    .and_then(|attr| serde_json::from_str(&attr.value).map_err(|_| ()))?;

                Ok(MyEvent { a, b })
            }
        }

        impl TypedEvent for MyEvent {}

        let t = MyEvent {
            a: Payload { x: 1, y: 2 },
            b: Payload { x: 3, y: 4 },
        };

        let e1 = Event::from(t.clone());
        // e2 is like e1 but with different indexing.
        let e2 = {
            let mut e = e1.clone();
            e.attributes[0].index = false;
            e.attributes[1].index = false;
            e
        };

        // Contract:

        // - `T::try_from(e) == Ok(t)` for all `t: T, e: Event` where
        //   `Event::from(t).eq_ignoring_index(e) == true`.
        assert_eq!(e1.eq_ignoring_index(&e2), true);
        assert_eq!(MyEvent::try_from(e1.clone()), Ok(t.clone()));
        assert_eq!(MyEvent::try_from(e2.clone()), Ok(t.clone()));

        // - `Event::from(T::try_from(e).unwrap()).eq_ignoring_index(e) == true` for all `e: Event`
        //   where `T::try_from(e)` returns `Ok(_)`.
        assert_eq!(
            Event::from(MyEvent::try_from(e1.clone()).unwrap()).eq_ignoring_index(&e1),
            true
        );
        assert_eq!(
            Event::from(MyEvent::try_from(e2.clone()).unwrap()).eq_ignoring_index(&e2),
            true
        );
    }
}
