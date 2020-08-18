//! Subscription- and subscription management-related functionality.

use crate::event::Event;
use crate::{Error, Id, Result};
use futures::task::{Context, Poll};
use futures::Stream;
use getrandom::getrandom;
use std::collections::HashMap;
use std::convert::TryInto;
use std::pin::Pin;
use tokio::sync::{mpsc, oneshot};

pub type EventRx = mpsc::Receiver<Result<Event>>;
pub type EventTx = mpsc::Sender<Result<Event>>;
pub type PendingResultTx = oneshot::Sender<Result<()>>;

/// An interface that can be used to asynchronously receive [`Event`]s for a
/// particular subscription.
///
/// ## Examples
///
/// ```
/// use tendermint_rpc::{SubscriptionId, Subscription};
/// use futures::StreamExt;
///
/// /// Prints `count` events from the given subscription.
/// async fn print_events(subs: &mut Subscription, count: usize) {
///     let mut counter = 0_usize;
///     while let Some(res) = subs.next().await {
///         // Technically, a subscription produces `Result<Event, Error>`
///         // instances. Errors can be produced by the remote endpoint at any
///         // time and need to be handled here.
///         let ev = res.unwrap();
///         println!("Got incoming event: {:?}", ev);
///         counter += 1;
///         if counter >= count {
///             break
///         }
///     }
/// }
/// ```
///
/// [`Event`]: ./event/struct.Event.html
#[derive(Debug)]
pub struct Subscription {
    /// The query for which events will be produced.
    pub query: String,
    /// The ID of this subscription (automatically assigned).
    pub id: SubscriptionId,
    event_rx: EventRx,
}

impl Stream for Subscription {
    type Item = Result<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.event_rx.poll_recv(cx)
    }
}

impl Subscription {
    pub fn new(id: SubscriptionId, query: String, event_rx: EventRx) -> Self {
        Self {
            id,
            query,
            event_rx,
        }
    }
}

/// Each new subscription is automatically assigned an ID.
///
/// By default, we generate random [UUIDv4] IDs for each subscription to
/// minimize chances of collision.
///
/// [UUIDv4]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_(random)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(String);

impl Default for SubscriptionId {
    fn default() -> Self {
        let mut bytes = [0; 16];
        getrandom(&mut bytes).expect("RNG failure!");

        let uuid = uuid::Builder::from_bytes(bytes)
            .set_variant(uuid::Variant::RFC4122)
            .set_version(uuid::Version::Random)
            .build();

        Self(uuid.to_string())
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<Id> for SubscriptionId {
    fn into(self) -> Id {
        Id::Str(self.0)
    }
}

impl TryInto<SubscriptionId> for Id {
    type Error = Error;

    fn try_into(self) -> std::result::Result<SubscriptionId, Self::Error> {
        match self {
            Id::Str(s) => Ok(SubscriptionId(s)),
            Id::Num(i) => Ok(SubscriptionId(format!("{}", i))),
            Id::None => Err(Error::client_error(
                "cannot convert an empty JSONRPC ID into a subscription ID",
            )),
        }
    }
}

#[derive(Debug)]
struct PendingSubscribe {
    query: String,
    event_tx: EventTx,
    result_tx: PendingResultTx,
}

#[derive(Debug)]
struct PendingUnsubscribe {
    subscription: Subscription,
    result_tx: PendingResultTx,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionState {
    Pending,
    Active,
    Cancelling,
    NotFound,
}

/// Provides a mechanism for tracking [`Subscription`]s and routing [`Event`]s
/// to those subscriptions.
///
/// This is useful when implementing your own RPC client transport layer.
///
/// [`Subscription`]: struct.Subscription.html
/// [`Event`]: ./event/struct.Event.html
///
#[derive(Debug)]
pub struct SubscriptionRouter {
    subscriptions: HashMap<String, HashMap<SubscriptionId, EventTx>>,
    pending_subscribe: HashMap<SubscriptionId, PendingSubscribe>,
    pending_unsubscribe: HashMap<SubscriptionId, PendingUnsubscribe>,
}

impl SubscriptionRouter {
    /// Publishes the given event to all of the subscriptions to which the
    /// event is relevant. At present, it matches purely based on the query
    /// associated with the event, and only queries that exactly match that of
    /// the event's.
    pub async fn publish(&mut self, ev: Event) {
        let subs_for_query = match self.subscriptions.get_mut(&ev.query) {
            Some(s) => s,
            None => return,
        };
        let mut disconnected = Vec::<SubscriptionId>::new();
        for (id, event_tx) in subs_for_query {
            // TODO(thane): Right now we automatically remove any disconnected
            //              or full channels. We must handle full channels
            //              differently to disconnected ones.
            if event_tx.send(Ok(ev.clone())).await.is_err() {
                disconnected.push(id.clone());
            }
        }
        let subs_for_query = self.subscriptions.get_mut(&ev.query).unwrap();
        for id in disconnected {
            subs_for_query.remove(&id);
        }
    }

    /// Immediately add a new subscription to the router without waiting for
    /// confirmation.
    pub fn add(&mut self, id: SubscriptionId, query: String, event_tx: EventTx) {
        let subs_for_query = match self.subscriptions.get_mut(&query) {
            Some(s) => s,
            None => {
                self.subscriptions.insert(query.clone(), HashMap::new());
                self.subscriptions.get_mut(&query).unwrap()
            }
        };
        subs_for_query.insert(id, event_tx);
    }

    /// Keep track of a pending subscription, which can either be confirmed or
    /// cancelled.
    pub fn add_pending_subscribe(
        &mut self,
        id: SubscriptionId,
        query: String,
        event_tx: EventTx,
        result_tx: PendingResultTx,
    ) {
        self.pending_subscribe.insert(
            id,
            PendingSubscribe {
                query,
                event_tx,
                result_tx,
            },
        );
    }

    /// Attempts to confirm the pending subscription with the given ID.
    ///
    /// Returns an error if it fails to respond (through the internal `oneshot`
    /// channel) to the original caller to indicate success.
    pub fn confirm_pending_subscribe(&mut self, id: &SubscriptionId) -> Result<()> {
        match self.pending_subscribe.remove(id) {
            Some(pending_subscribe) => {
                self.add(
                    id.clone(),
                    pending_subscribe.query.clone(),
                    pending_subscribe.event_tx,
                );
                Ok(pending_subscribe.result_tx.send(Ok(())).map_err(|_| {
                    Error::client_error(format!(
                        "failed to communicate result of pending subscription with ID: {}",
                        id
                    ))
                })?)
            }
            None => Ok(()),
        }
    }

    /// Attempts to cancel the pending subscription with the given ID, sending
    /// the specified error to the original creator of the attempted
    /// subscription.
    pub fn cancel_pending_subscribe(
        &mut self,
        id: &SubscriptionId,
        err: impl Into<Error>,
    ) -> Result<()> {
        match self.pending_subscribe.remove(id) {
            Some(pending_subscribe) => Ok(pending_subscribe
                .result_tx
                .send(Err(err.into()))
                .map_err(|_| {
                    Error::client_error(format!(
                        "failed to communicate result of pending subscription with ID: {}",
                        id
                    ))
                })?),
            None => Ok(()),
        }
    }

    /// Immediately remove the given subscription and consume it.
    pub fn remove(&mut self, subs: Subscription) {
        let subs_for_query = match self.subscriptions.get_mut(&subs.query) {
            Some(s) => s,
            None => return,
        };
        subs_for_query.remove(&subs.id);
    }

    /// Keeps track of a pending unsubscribe request, which can either be
    /// confirmed or cancelled.
    pub fn add_pending_unsubscribe(&mut self, subs: Subscription, result_tx: PendingResultTx) {
        self.pending_unsubscribe.insert(
            subs.id.clone(),
            PendingUnsubscribe {
                subscription: subs,
                result_tx,
            },
        );
    }

    /// Confirm the pending unsubscribe request for the subscription with the
    /// given ID.
    pub fn confirm_pending_unsubscribe(&mut self, id: &SubscriptionId) -> Result<()> {
        match self.pending_unsubscribe.remove(id) {
            Some(pending_unsubscribe) => {
                let (subscription, result_tx) = (
                    pending_unsubscribe.subscription,
                    pending_unsubscribe.result_tx,
                );
                self.remove(subscription);
                Ok(result_tx.send(Ok(())).map_err(|_| {
                    Error::client_error(format!(
                        "failed to communicate result of pending unsubscribe for subscription with ID: {}",
                        id
                    ))
                })?)
            }
            None => Ok(()),
        }
    }

    /// Cancel the pending unsubscribe request for the subscription with the
    /// given ID, responding with the given error.
    pub fn cancel_pending_unsubscribe(
        &mut self,
        id: &SubscriptionId,
        err: impl Into<Error>,
    ) -> Result<()> {
        match self.pending_unsubscribe.remove(id) {
            Some(pending_unsubscribe) => {
                Ok(pending_unsubscribe.result_tx.send(Err(err.into())).map_err(|_| {
                    Error::client_error(format!(
                        "failed to communicate result of pending unsubscribe for subscription with ID: {}",
                        id
                    ))
                })?)
            }
            None => Ok(()),
        }
    }

    pub fn is_active(&self, id: &SubscriptionId) -> bool {
        self.subscriptions
            .iter()
            .any(|(_query, subs_for_query)| subs_for_query.contains_key(id))
    }

    pub fn get_active_subscription_mut(&mut self, id: &SubscriptionId) -> Option<&mut EventTx> {
        self.subscriptions
            .iter_mut()
            .find(|(_query, subs_for_query)| subs_for_query.contains_key(id))
            .and_then(|(_query, subs_for_query)| subs_for_query.get_mut(id))
    }

    /// Utility method to determine the current state of the subscription with
    /// the given ID.
    pub fn subscription_state(&self, id: &SubscriptionId) -> SubscriptionState {
        if self.pending_subscribe.contains_key(id) {
            return SubscriptionState::Pending;
        }
        if self.pending_unsubscribe.contains_key(id) {
            return SubscriptionState::Cancelling;
        }
        if self.is_active(id) {
            return SubscriptionState::Active;
        }
        SubscriptionState::NotFound
    }
}

impl Default for SubscriptionRouter {
    fn default() -> Self {
        Self {
            subscriptions: HashMap::new(),
            pending_subscribe: HashMap::new(),
            pending_unsubscribe: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::event::{Event, WrappedEvent};
    use std::path::PathBuf;
    use tokio::fs;

    async fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json"))
            .await
            .unwrap()
    }

    async fn read_event(name: &str) -> Event {
        serde_json::from_str::<WrappedEvent>(read_json_fixture(name).await.as_str())
            .unwrap()
            .into_result()
            .unwrap()
    }

    #[tokio::test]
    async fn router_basic_pub_sub() {
        let mut router = SubscriptionRouter::default();

        let (subs1_id, subs2_id, subs3_id) = (
            SubscriptionId::default(),
            SubscriptionId::default(),
            SubscriptionId::default(),
        );
        let (subs1_event_tx, mut subs1_event_rx) = mpsc::channel(1);
        let (subs2_event_tx, mut subs2_event_rx) = mpsc::channel(1);
        let (subs3_event_tx, mut subs3_event_rx) = mpsc::channel(1);

        // Two subscriptions with the same query
        router.add(subs1_id.clone(), "query1".into(), subs1_event_tx);
        router.add(subs2_id.clone(), "query1".into(), subs2_event_tx);
        // Another subscription with a different query
        router.add(subs3_id.clone(), "query2".into(), subs3_event_tx);

        let mut ev = read_event("event_new_block_1").await;
        ev.query = "query1".into();
        router.publish(ev.clone()).await;

        let subs1_ev = subs1_event_rx.try_recv().unwrap().unwrap();
        let subs2_ev = subs2_event_rx.try_recv().unwrap().unwrap();
        if subs3_event_rx.try_recv().is_ok() {
            panic!("should not have received an event here");
        }
        assert_eq!(ev, subs1_ev);
        assert_eq!(ev, subs2_ev);

        ev.query = "query2".into();
        router.publish(ev.clone()).await;

        if subs1_event_rx.try_recv().is_ok() {
            panic!("should not have received an event here");
        }
        if subs2_event_rx.try_recv().is_ok() {
            panic!("should not have received an event here");
        }
        let subs3_ev = subs3_event_rx.try_recv().unwrap().unwrap();
        assert_eq!(ev, subs3_ev);
    }

    #[tokio::test]
    async fn router_pending_subscription() {
        let mut router = SubscriptionRouter::default();
        let subs_id = SubscriptionId::default();
        let (event_tx, mut event_rx) = mpsc::channel(1);
        let (result_tx, mut result_rx) = oneshot::channel();
        let query = "query".to_string();
        let mut ev = read_event("event_new_block_1").await;
        ev.query = query.clone();

        assert_eq!(
            SubscriptionState::NotFound,
            router.subscription_state(&subs_id)
        );
        router.add_pending_subscribe(subs_id.clone(), query.clone(), event_tx, result_tx);
        assert_eq!(
            SubscriptionState::Pending,
            router.subscription_state(&subs_id)
        );
        router.publish(ev.clone()).await;
        if event_rx.try_recv().is_ok() {
            panic!("should not have received an event prior to confirming a pending subscription")
        }

        router.confirm_pending_subscribe(&subs_id).unwrap();
        assert_eq!(
            SubscriptionState::Active,
            router.subscription_state(&subs_id)
        );
        if event_rx.try_recv().is_ok() {
            panic!("should not have received an event here")
        }
        if result_rx.try_recv().is_err() {
            panic!("we should have received successful confirmation of the new subscription")
        }

        router.publish(ev.clone()).await;
        let received_ev = event_rx.try_recv().unwrap().unwrap();
        assert_eq!(ev, received_ev);

        let (result_tx, mut result_rx) = oneshot::channel();
        router.add_pending_unsubscribe(
            Subscription::new(subs_id.clone(), query.clone(), event_rx),
            result_tx,
        );
        assert_eq!(
            SubscriptionState::Cancelling,
            router.subscription_state(&subs_id),
        );

        router.confirm_pending_unsubscribe(&subs_id).unwrap();
        assert_eq!(
            SubscriptionState::NotFound,
            router.subscription_state(&subs_id)
        );
        router.publish(ev.clone()).await;
        if result_rx.try_recv().is_err() {
            panic!("we should have received successful confirmation of the unsubscribe request")
        }
    }

    #[tokio::test]
    async fn router_cancel_pending_subscription() {
        let mut router = SubscriptionRouter::default();
        let subs_id = SubscriptionId::default();
        let (event_tx, mut event_rx) = mpsc::channel(1);
        let (result_tx, mut result_rx) = oneshot::channel();
        let query = "query".to_string();
        let mut ev = read_event("event_new_block_1").await;
        ev.query = query.clone();

        assert_eq!(
            SubscriptionState::NotFound,
            router.subscription_state(&subs_id)
        );
        router.add_pending_subscribe(subs_id.clone(), query, event_tx, result_tx);
        assert_eq!(
            SubscriptionState::Pending,
            router.subscription_state(&subs_id)
        );
        router.publish(ev.clone()).await;
        if event_rx.try_recv().is_ok() {
            panic!("should not have received an event prior to confirming a pending subscription")
        }

        let cancel_error = Error::client_error("cancelled");
        router
            .cancel_pending_subscribe(&subs_id, cancel_error.clone())
            .unwrap();
        assert_eq!(
            SubscriptionState::NotFound,
            router.subscription_state(&subs_id)
        );
        assert_eq!(Err(cancel_error), result_rx.try_recv().unwrap());

        router.publish(ev.clone()).await;
        if event_rx.try_recv().is_ok() {
            panic!("should not have received an event prior to confirming a pending subscription")
        }
    }
}
