//! Subscription- and subscription management-related functionality.

use crate::event::Event;
use futures::task::{Context, Poll};
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::sync::mpsc;

/// An interface that can be used to asynchronously receive events for a
/// particular subscription.
#[derive(Debug)]
pub struct Subscription {
    pub query: String,
    id: SubscriptionId,
    event_rx: mpsc::Receiver<Event>,
}

impl Stream for Subscription {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.event_rx.poll_recv(cx)
    }
}

impl Subscription {
    pub fn new(id: SubscriptionId, query: String, event_rx: mpsc::Receiver<Event>) -> Self {
        Self {
            id,
            query,
            event_rx,
        }
    }
}

/// Each new subscription is automatically assigned an ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(usize);

impl From<usize> for SubscriptionId {
    fn from(u: usize) -> Self {
        SubscriptionId(u)
    }
}

impl From<SubscriptionId> for usize {
    fn from(subs_id: SubscriptionId) -> Self {
        subs_id.0
    }
}

impl SubscriptionId {
    pub fn next(&self) -> SubscriptionId {
        SubscriptionId(self.0 + 1)
    }
}

/// Provides a mechanism for tracking subscriptions and routing events to those
/// subscriptions. This is useful when implementing your own RPC client
/// transport layer.
#[derive(Debug, Clone)]
pub struct SubscriptionRouter(HashMap<String, HashMap<SubscriptionId, mpsc::Sender<Event>>>);

impl SubscriptionRouter {
    /// Create a new empty `SubscriptionRouter`.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Publishes the given event to all of the subscriptions to which the
    /// event is relevant. At present, it matches purely based on the query
    /// associated with the event, and only queries that exactly match that of
    /// the event's.
    pub async fn publish(&mut self, ev: Event) {
        let subs_for_query = match self.0.get_mut(&ev.query) {
            Some(s) => s,
            None => return,
        };
        let mut disconnected = Vec::<SubscriptionId>::new();
        for (id, event_tx) in subs_for_query {
            // TODO(thane): Right now we automatically remove any disconnected
            //              or full channels. We must handle full channels
            //              differently to disconnected ones.
            if let Err(_) = event_tx.send(ev.clone()).await {
                disconnected.push(id.clone());
            }
        }
        let subs_for_query = self.0.get_mut(&ev.query).unwrap();
        for id in disconnected {
            subs_for_query.remove(&id);
        }
    }

    /// Keep track of a new subscription for a particular query.
    pub fn add(&mut self, id: SubscriptionId, query: String, event_tx: mpsc::Sender<Event>) {
        let subs_for_query = match self.0.get_mut(&query) {
            Some(s) => s,
            None => {
                self.0.insert(query.clone(), HashMap::new());
                self.0.get_mut(&query).unwrap()
            }
        };
        subs_for_query.insert(id, event_tx);
    }

    /// Remove the given subscription and consume it.
    pub fn remove(&mut self, subs: Subscription) {
        let subs_for_query = match self.0.get_mut(&subs.query) {
            Some(s) => s,
            None => return,
        };
        subs_for_query.remove(&subs.id);
    }
}
