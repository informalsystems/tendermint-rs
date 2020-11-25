//! Event routing for subscriptions.

use crate::client::subscription::SubscriptionTx;
use crate::event::Event;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Provides a mechanism for tracking [`Subscription`]s and routing [`Event`]s
/// to those subscriptions.
///
/// [`Subscription`]: struct.Subscription.html
/// [`Event`]: ./event/struct.Event.html
#[derive(Debug)]
pub struct SubscriptionRouter {
    // A map of subscription queries to collections of subscription IDs and
    // their result channels. Used for publishing events relating to a specific
    // query.
    subscriptions: HashMap<String, HashMap<String, SubscriptionTx>>,
}

impl SubscriptionRouter {
    /// Publishes the given event to all of the subscriptions to which the
    /// event is relevant. At present, it matches purely based on the query
    /// associated with the event, and only queries that exactly match that of
    /// the event's.
    pub fn publish(&mut self, ev: &Event) -> PublishResult {
        let subs_for_query = match self.subscriptions.get_mut(&ev.query) {
            Some(s) => s,
            None => return PublishResult::NoSubscribers,
        };
        // We assume here that any failure to publish an event is an indication
        // that the receiver end of the channel has been dropped, which allows
        // us to safely stop tracking the subscription.
        let mut disconnected = HashSet::new();
        for (id, event_tx) in subs_for_query.borrow_mut() {
            if let Err(e) = event_tx.send(Ok(ev.clone())) {
                disconnected.insert(id.clone());
                debug!(
                    "Automatically disconnecting subscription with ID {} for query \"{}\" due to failure to publish to it: {}",
                    id, ev.query, e
                );
            }
        }
        for id in disconnected {
            subs_for_query.remove(&id);
        }
        if subs_for_query.is_empty() {
            PublishResult::AllDisconnected
        } else {
            PublishResult::Success
        }
    }

    /// Immediately add a new subscription to the router without waiting for
    /// confirmation.
    pub fn add(&mut self, id: impl ToString, query: impl ToString, tx: SubscriptionTx) {
        let query = query.to_string();
        let subs_for_query = match self.subscriptions.get_mut(&query) {
            Some(s) => s,
            None => {
                self.subscriptions.insert(query.clone(), HashMap::new());
                self.subscriptions.get_mut(&query).unwrap()
            }
        };
        subs_for_query.insert(id.to_string(), tx);
    }

    /// Removes all the subscriptions relating to the given query.
    pub fn remove_by_query(&mut self, query: impl ToString) -> usize {
        self.subscriptions
            .remove(&query.to_string())
            .map(|subs_for_query| subs_for_query.len())
            .unwrap_or(0)
    }
}

#[cfg(feature = "websocket-client")]
impl SubscriptionRouter {
    /// Returns the number of active subscriptions for the given query.
    pub fn num_subscriptions_for_query(&self, query: impl ToString) -> usize {
        self.subscriptions
            .get(&query.to_string())
            .map(|subs_for_query| subs_for_query.len())
            .unwrap_or(0)
    }
}

impl Default for SubscriptionRouter {
    fn default() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PublishResult {
    Success,
    NoSubscribers,
    AllDisconnected,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::client::sync::{unbounded, ChannelRx};
    use crate::event::{Event, WrappedEvent};
    use crate::utils::uuid_str;
    use std::path::PathBuf;
    use tokio::fs;
    use tokio::time::{self, Duration};

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

    async fn must_recv<T>(ch: &mut ChannelRx<T>, timeout_ms: u64) -> T {
        let mut delay = time::sleep(Duration::from_millis(timeout_ms));
        tokio::select! {
            _ = &mut delay, if !delay.is_elapsed() => panic!("timed out waiting for recv"),
            Some(v) = ch.recv() => v,
        }
    }

    async fn must_not_recv<T>(ch: &mut ChannelRx<T>, timeout_ms: u64)
    where
        T: std::fmt::Debug,
    {
        let mut delay = time::sleep(Duration::from_millis(timeout_ms));
        tokio::select! {
            _ = &mut delay, if !delay.is_elapsed() => (),
            Some(v) = ch.recv() => panic!("got unexpected result from channel: {:?}", v),
        }
    }

    #[tokio::test]
    async fn router_basic_pub_sub() {
        let mut router = SubscriptionRouter::default();

        let (subs1_id, subs2_id, subs3_id) = (uuid_str(), uuid_str(), uuid_str());
        let (subs1_event_tx, mut subs1_event_rx) = unbounded();
        let (subs2_event_tx, mut subs2_event_rx) = unbounded();
        let (subs3_event_tx, mut subs3_event_rx) = unbounded();

        // Two subscriptions with the same query
        router.add(subs1_id, "query1", subs1_event_tx);
        router.add(subs2_id, "query1", subs2_event_tx);
        // Another subscription with a different query
        router.add(subs3_id, "query2", subs3_event_tx);

        let mut ev = read_event("event_new_block_1").await;
        ev.query = "query1".into();
        router.publish(&ev);

        let subs1_ev = must_recv(&mut subs1_event_rx, 500).await.unwrap();
        let subs2_ev = must_recv(&mut subs2_event_rx, 500).await.unwrap();
        must_not_recv(&mut subs3_event_rx, 50).await;
        assert_eq!(ev, subs1_ev);
        assert_eq!(ev, subs2_ev);

        ev.query = "query2".into();
        router.publish(&ev);

        must_not_recv(&mut subs1_event_rx, 50).await;
        must_not_recv(&mut subs2_event_rx, 50).await;
        let subs3_ev = must_recv(&mut subs3_event_rx, 500).await.unwrap();
        assert_eq!(ev, subs3_ev);
    }
}
