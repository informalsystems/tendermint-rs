//! Event routing for subscriptions.

use core::str::FromStr;

use alloc::collections::{BTreeMap as HashMap, BTreeSet as HashSet};

use tracing::debug;

use crate::client::subscription::SubscriptionTx;
use crate::error::Error;
use crate::event::Event;
use crate::prelude::*;
use crate::query::Query;

pub type SubscriptionQuery = Query;
pub type SubscriptionId = String;

#[cfg_attr(not(feature = "websocket"), allow(dead_code))]
pub type SubscriptionIdRef<'a> = &'a str;

/// Provides a mechanism for tracking [`Subscription`]s and routing [`Event`]s
/// to those subscriptions.
///
/// [`Subscription`]: struct.Subscription.html
/// [`Event`]: ./event/struct.Event.html
#[derive(Debug, Default)]
pub struct SubscriptionRouter {
    /// A map of subscription queries to collections of subscription IDs and
    /// their result channels. Used for publishing events relating to a specific
    /// query.
    subscriptions: HashMap<SubscriptionQuery, HashMap<SubscriptionId, SubscriptionTx>>,
}

impl SubscriptionRouter {
    /// Publishes the given error to all of the subscriptions to which the
    /// error is relevant, based on the given subscription id query.
    #[cfg_attr(not(feature = "websocket"), allow(dead_code))]
    pub fn publish_error(&mut self, id: SubscriptionIdRef<'_>, err: Error) -> PublishResult {
        if let Some(query) = self.subscription_query(id).cloned() {
            self.publish(query, Err(err))
        } else {
            PublishResult::NoSubscribers
        }
    }

    /// Get the query associated with the given subscription.
    #[cfg_attr(not(feature = "websocket"), allow(dead_code))]
    fn subscription_query(&self, id: SubscriptionIdRef<'_>) -> Option<&SubscriptionQuery> {
        for (query, subs) in &self.subscriptions {
            if subs.contains_key(id) {
                return Some(query);
            }
        }

        None
    }

    /// Publishes the given event to all of the subscriptions to which the
    /// event is relevant, based on the associated query.
    #[cfg_attr(not(feature = "websocket"), allow(dead_code))]
    pub fn publish_event(&mut self, ev: Event) -> PublishResult {
        let query = match Query::from_str(&ev.query) {
            Ok(query) => query,
            Err(e) => {
                return PublishResult::Error(format!(
                    "Failed to parse query from event: {:?}, reason: {e}",
                    ev.query
                ));
            },
        };

        self.publish(query, Ok(ev))
    }

    /// Publishes the given event/error to all of the subscriptions to which the
    /// event/error is relevant, based on the given query.
    pub fn publish(&mut self, query: SubscriptionQuery, ev: Result<Event, Error>) -> PublishResult {
        let subs_for_query = match self.subscriptions.get_mut(&query) {
            Some(s) => s,
            None => return PublishResult::NoSubscribers,
        };

        // We assume here that any failure to publish an event is an indication
        // that the receiver end of the channel has been dropped, which allows
        // us to safely stop tracking the subscription.
        let mut disconnected = HashSet::new();
        for (id, event_tx) in subs_for_query.iter_mut() {
            if let Err(e) = event_tx.send(ev.clone()) {
                disconnected.insert(id.clone());
                debug!(
                    "Automatically disconnecting subscription with ID {} for query \"{}\" due to failure to publish to it: {}",
                    id, query, e
                );
            }
        }

        for id in disconnected {
            subs_for_query.remove(&id);
        }

        if subs_for_query.is_empty() {
            PublishResult::AllDisconnected(query)
        } else {
            PublishResult::Success
        }
    }

    /// Immediately add a new subscription to the router without waiting for
    /// confirmation.
    pub fn add(&mut self, id: impl ToString, query: SubscriptionQuery, tx: SubscriptionTx) {
        let subs_for_query = self.subscriptions.entry(query).or_default();
        subs_for_query.insert(id.to_string(), tx);
    }

    /// Removes all the subscriptions relating to the given query.
    pub fn remove_by_query(&mut self, query: &SubscriptionQuery) -> usize {
        self.subscriptions
            .remove(query)
            .map(|subs_for_query| subs_for_query.len())
            .unwrap_or(0)
    }
}

#[cfg(feature = "websocket-client")]
impl SubscriptionRouter {
    /// Returns the number of active subscriptions for the given query.
    pub fn num_subscriptions_for_query(&self, query: &SubscriptionQuery) -> usize {
        self.subscriptions
            .get(query)
            .map(|subs_for_query| subs_for_query.len())
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub enum PublishResult {
    Success,
    NoSubscribers,
    // All subscriptions for the given query have disconnected.
    AllDisconnected(SubscriptionQuery),
    Error(String),
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use tokio::{
        fs,
        time::{self, Duration},
    };

    use super::*;
    use crate::{
        client::sync::{unbounded, ChannelRx},
        event::Event,
        utils::uuid_str,
    };

    async fn read_json_fixture(version: &str, name: &str) -> String {
        fs::read_to_string(
            PathBuf::from("./tests/kvstore_fixtures")
                .join(version)
                .join("incoming")
                .join(name.to_owned() + ".json"),
        )
        .await
        .unwrap()
    }

    async fn must_recv<T>(ch: &mut ChannelRx<T>, timeout_ms: u64) -> T {
        let delay = time::sleep(Duration::from_millis(timeout_ms));
        tokio::select! {
            _ = delay, if !delay.is_elapsed() => panic!("timed out waiting for recv"),
            Some(v) = ch.recv() => v,
        }
    }

    async fn must_not_recv<T>(ch: &mut ChannelRx<T>, timeout_ms: u64)
    where
        T: core::fmt::Debug,
    {
        let delay = time::sleep(Duration::from_millis(timeout_ms));
        tokio::select! {
            _ = delay, if !delay.is_elapsed() => (),
            Some(v) = ch.recv() => panic!("got unexpected result from channel: {:?}", v),
        }
    }

    async fn test_router_basic_pub_sub(mut ev: Event) {
        let mut router = SubscriptionRouter::default();

        let (subs1_id, subs2_id, subs3_id) = (uuid_str(), uuid_str(), uuid_str());
        let (subs1_event_tx, mut subs1_event_rx) = unbounded();
        let (subs2_event_tx, mut subs2_event_rx) = unbounded();
        let (subs3_event_tx, mut subs3_event_rx) = unbounded();

        let query1: Query = "tm.event = 'Tx'".parse().unwrap();
        let query2: Query = "tm.event = 'NewBlock'".parse().unwrap();

        // Two subscriptions with the same query
        router.add(subs1_id, query1.clone(), subs1_event_tx);
        router.add(subs2_id, query1.clone(), subs2_event_tx);
        // Another subscription with a different query
        router.add(subs3_id, query2.clone(), subs3_event_tx);

        ev.query = query1.to_string();
        router.publish_event(ev.clone());

        let subs1_ev = must_recv(&mut subs1_event_rx, 500).await.unwrap();
        let subs2_ev = must_recv(&mut subs2_event_rx, 500).await.unwrap();
        must_not_recv(&mut subs3_event_rx, 50).await;
        assert_eq!(ev, subs1_ev);
        assert_eq!(ev, subs2_ev);

        ev.query = query2.to_string();
        router.publish_event(ev.clone());

        must_not_recv(&mut subs1_event_rx, 50).await;
        must_not_recv(&mut subs2_event_rx, 50).await;
        let subs3_ev = must_recv(&mut subs3_event_rx, 500).await.unwrap();
        assert_eq!(ev, subs3_ev);
    }

    async fn test_router_pub_sub_diff_event_type_format(mut ev: Event) {
        let mut router = SubscriptionRouter::default();

        let subs1_id = uuid_str();
        let (subs1_event_tx, mut subs1_event_rx) = unbounded();

        let query1: Query = "tm.event = 'Tx'".parse().unwrap();
        router.add(subs1_id, query1.clone(), subs1_event_tx);

        // Query is equivalent but formatted slightly differently
        ev.query = "tm.event='Tx'".to_string();
        router.publish_event(ev.clone());

        let subs1_ev = must_recv(&mut subs1_event_rx, 500).await.unwrap();
        assert_eq!(ev, subs1_ev);
    }

    async fn test_router_pub_sub_two_eq_queries_diff_format(mut ev1: Event, mut ev2: Event) {
        let mut router = SubscriptionRouter::default();

        let (subs1_id, subs2_id, subs3_id) = (uuid_str(), uuid_str(), uuid_str());
        let (subs1_event_tx, mut subs1_event_rx) = unbounded();
        let (subs2_event_tx, mut subs2_event_rx) = unbounded();
        let (subs3_event_tx, mut subs3_event_rx) = unbounded();

        let query1: Query =
            "tm.event = 'Tx' AND message.module = 'ibc_client' AND message.foo = 'bar'"
                .parse()
                .unwrap();
        let query2: Query =
            "message.module = 'ibc_client' AND message.foo = 'bar' AND tm.event = 'Tx'"
                .parse()
                .unwrap();

        assert_eq!(query1, query2);

        let query3: Query = "tm.event = 'NewBlock'".parse().unwrap();

        router.add(subs1_id, query1.clone(), subs1_event_tx);
        router.add(subs2_id, query2.clone(), subs2_event_tx);
        router.add(subs3_id, query3.clone(), subs3_event_tx);

        std::dbg!(&router);

        // Queries are equivalent but formatted slightly differently
        ev1.query =
            "tm.event='Tx' AND message.module='ibc_client' AND message.foo='bar'".to_string();
        router.publish_event(ev1.clone());

        ev2.query =
            "message.module='ibc_client' AND message.foo='bar' AND tm.event='Tx'".to_string();
        router.publish_event(ev2.clone());

        let subs1_ev1 = must_recv(&mut subs1_event_rx, 500).await.unwrap();
        assert_eq!(ev1, subs1_ev1);
        let subs2_ev1 = must_recv(&mut subs2_event_rx, 500).await.unwrap();
        assert_eq!(ev1, subs2_ev1);

        let subs1_ev2 = must_recv(&mut subs1_event_rx, 500).await.unwrap();
        assert_eq!(ev2, subs1_ev2);
        let subs2_ev2 = must_recv(&mut subs2_event_rx, 500).await.unwrap();
        assert_eq!(ev2, subs2_ev2);

        must_not_recv(&mut subs3_event_rx, 50).await;
    }

    mod v0_34 {
        use super::*;

        type WrappedEvent = crate::response::Wrapper<crate::event::v0_34::DialectEvent>;

        async fn read_event(name: &str) -> Event {
            serde_json::from_str::<WrappedEvent>(read_json_fixture("v0_34", name).await.as_str())
                .unwrap()
                .into_result()
                .unwrap()
                .into()
        }

        #[tokio::test]
        async fn router_basic_pub_sub() {
            test_router_basic_pub_sub(read_event("subscribe_newblock_0").await).await
        }

        #[tokio::test]
        async fn router_pub_sub_diff_event_type_format() {
            test_router_pub_sub_diff_event_type_format(read_event("subscribe_newblock_0").await)
                .await
        }

        #[tokio::test]
        async fn router_pub_sub_two_eq_queries_diff_format() {
            test_router_pub_sub_two_eq_queries_diff_format(
                read_event("subscribe_newblock_0").await,
                read_event("subscribe_newblock_1").await,
            )
            .await
        }
    }

    mod v0_37 {
        use super::*;

        type WrappedEvent = crate::response::Wrapper<crate::event::v0_37::DeEvent>;

        async fn read_event(name: &str) -> Event {
            serde_json::from_str::<WrappedEvent>(read_json_fixture("v0_37", name).await.as_str())
                .unwrap()
                .into_result()
                .unwrap()
                .into()
        }

        #[tokio::test]
        async fn router_basic_pub_sub() {
            test_router_basic_pub_sub(read_event("subscribe_newblock_0").await).await
        }

        #[tokio::test]
        async fn router_pub_sub_diff_event_type_format() {
            test_router_pub_sub_diff_event_type_format(read_event("subscribe_newblock_0").await)
                .await
        }

        #[tokio::test]
        async fn router_pub_sub_two_eq_queries_diff_format() {
            test_router_pub_sub_two_eq_queries_diff_format(
                read_event("subscribe_newblock_0").await,
                read_event("subscribe_newblock_1").await,
            )
            .await
        }
    }
}
