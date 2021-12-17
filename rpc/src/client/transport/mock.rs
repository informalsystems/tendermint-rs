//! Mock client implementation for use in testing.

use crate::client::subscription::SubscriptionTx;
use crate::client::sync::{unbounded, ChannelRx, ChannelTx};
use crate::client::transport::router::SubscriptionRouter;
use crate::event::Event;
use crate::prelude::*;
use crate::query::Query;
use crate::utils::uuid_str;
use crate::{Client, Error, Method, Request, Response, Subscription, SubscriptionClient};
use alloc::collections::BTreeMap as HashMap;
use async_trait::async_trait;

/// A mock client implementation for use in testing.
///
/// ## Examples
///
/// ```rust
/// use tendermint_rpc::{Client, Method, MockClient, MockRequestMatcher, MockRequestMethodMatcher};
///
/// const ABCI_INFO_RESPONSE: &str = r#"{
///   "jsonrpc": "2.0",
///   "id": "",
///   "result": {
///     "response": {
///       "data": "GaiaApp",
///       "version": "0.17.0",
///       "app_version": "1",
///       "last_block_height": "488120",
///       "last_block_app_hash": "2LnCw0fN+Zq/gs5SOuya/GRHUmtWftAqAkTUuoxl4g4="
///     }
///   }
/// }"#;
///
/// tokio_test::block_on(async {
///     let matcher = MockRequestMethodMatcher::default()
///         .map(Method::AbciInfo, Ok(ABCI_INFO_RESPONSE.to_string()));
///     let (client, driver) = MockClient::new(matcher);
///     let driver_hdl = tokio::spawn(async move { driver.run().await });
///
///     let abci_info = client.abci_info().await.unwrap();
///     println!("Got mock ABCI info: {:?}", abci_info);
///     assert_eq!("GaiaApp".to_string(), abci_info.data);
///
///     client.close();
///     driver_hdl.await.unwrap();
/// });
/// ```
#[derive(Debug)]
pub struct MockClient<M: MockRequestMatcher> {
    matcher: M,
    driver_tx: ChannelTx<DriverCommand>,
}

#[async_trait]
impl<M: MockRequestMatcher> Client for MockClient<M> {
    async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: Request,
    {
        self.matcher
            .response_for(request)
            .ok_or_else(Error::mismatch_response)?
    }
}

impl<M: MockRequestMatcher> MockClient<M> {
    /// Create a new mock RPC client using the given request matcher.
    pub fn new(matcher: M) -> (Self, MockClientDriver) {
        let (driver_tx, driver_rx) = unbounded();
        (
            Self { matcher, driver_tx },
            MockClientDriver::new(driver_rx),
        )
    }

    /// Publishes the given event to all subscribers whose query exactly
    /// matches that of the event.
    pub fn publish(&self, ev: &Event) {
        self.driver_tx
            .send(DriverCommand::Publish(Box::new(ev.clone())))
            .unwrap();
    }

    /// Signal to the mock client's driver to terminate.
    pub fn close(self) {
        self.driver_tx.send(DriverCommand::Terminate).unwrap();
    }
}

#[async_trait]
impl<M: MockRequestMatcher> SubscriptionClient for MockClient<M> {
    async fn subscribe(&self, query: Query) -> Result<Subscription, Error> {
        let id = uuid_str();
        let (subs_tx, subs_rx) = unbounded();
        let (result_tx, mut result_rx) = unbounded();
        self.driver_tx.send(DriverCommand::Subscribe {
            id: id.clone(),
            query: query.clone(),
            subscription_tx: subs_tx,
            result_tx,
        })?;
        result_rx.recv().await.unwrap()?;
        Ok(Subscription::new(id, query, subs_rx))
    }

    async fn unsubscribe(&self, query: Query) -> Result<(), Error> {
        let (result_tx, mut result_rx) = unbounded();
        self.driver_tx
            .send(DriverCommand::Unsubscribe { query, result_tx })?;
        result_rx.recv().await.unwrap()
    }

    fn close(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum DriverCommand {
    Subscribe {
        id: String,
        query: Query,
        subscription_tx: SubscriptionTx,
        result_tx: ChannelTx<Result<(), Error>>,
    },
    Unsubscribe {
        query: Query,
        result_tx: ChannelTx<Result<(), Error>>,
    },
    Publish(Box<Event>),
    Terminate,
}

#[derive(Debug)]
pub struct MockClientDriver {
    router: SubscriptionRouter,
    rx: ChannelRx<DriverCommand>,
}

impl MockClientDriver {
    pub fn new(rx: ChannelRx<DriverCommand>) -> Self {
        Self {
            router: SubscriptionRouter::default(),
            rx,
        }
    }

    pub async fn run(mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
            Some(cmd) = self.rx.recv() => match cmd {
                    DriverCommand::Subscribe { id, query, subscription_tx, result_tx } => {
                        self.subscribe(id, query, subscription_tx, result_tx);
                    }
                    DriverCommand::Unsubscribe { query, result_tx } => {
                        self.unsubscribe(query, result_tx);
                    }
                    DriverCommand::Publish(event) => self.publish(*event),
                    DriverCommand::Terminate => return Ok(()),
                }
            }
        }
    }

    fn subscribe(
        &mut self,
        id: String,
        query: Query,
        subscription_tx: SubscriptionTx,
        result_tx: ChannelTx<Result<(), Error>>,
    ) {
        self.router.add(id, query, subscription_tx);
        result_tx.send(Ok(())).unwrap();
    }

    fn unsubscribe(&mut self, query: Query, result_tx: ChannelTx<Result<(), Error>>) {
        self.router.remove_by_query(query);
        result_tx.send(Ok(())).unwrap();
    }

    fn publish(&mut self, event: Event) {
        self.router.publish_event(event);
    }
}

/// A trait required by the [`MockClient`] that allows for different approaches
/// to mocking responses for specific requests.
///
/// [`MockClient`]: struct.MockClient.html
pub trait MockRequestMatcher: Send + Sync {
    /// Provide the corresponding response for the given request (if any).
    fn response_for<R>(&self, request: R) -> Option<Result<R::Response, Error>>
    where
        R: Request;
}

/// Provides a simple [`MockRequestMatcher`] implementation that simply maps
/// requests with specific methods to responses.
///
/// [`MockRequestMatcher`]: trait.MockRequestMatcher.html
#[derive(Debug, Default)]
pub struct MockRequestMethodMatcher {
    mappings: HashMap<Method, Result<String, Error>>,
}

impl MockRequestMatcher for MockRequestMethodMatcher {
    fn response_for<R>(&self, request: R) -> Option<Result<R::Response, Error>>
    where
        R: Request,
    {
        self.mappings.get(&request.method()).map(|res| match res {
            Ok(json) => R::Response::from_string(json),
            Err(e) => Err(e.clone()),
        })
    }
}

impl MockRequestMethodMatcher {
    /// Maps all incoming requests with the given method such that their
    /// corresponding response will be `response`.
    ///
    /// Successful responses must be JSON-encoded.
    #[allow(dead_code)]
    pub fn map(mut self, method: Method, response: Result<String, Error>) -> Self {
        self.mappings.insert(method, response);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::query::EventType;
    use futures::StreamExt;
    use std::path::PathBuf;
    use tendermint::block::Height;
    use tendermint::chain::Id;
    use tokio::fs;

    async fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(
            PathBuf::from("./tests/kvstore_fixtures/incoming/").join(name.to_owned() + ".json"),
        )
        .await
        .unwrap()
    }

    async fn read_event(name: &str) -> Event {
        Event::from_string(&read_json_fixture(name).await).unwrap()
    }

    #[tokio::test]
    async fn mock_client() {
        let abci_info_fixture = read_json_fixture("abci_info").await;
        let block_fixture = read_json_fixture("block_at_height_10").await;
        let matcher = MockRequestMethodMatcher::default()
            .map(Method::AbciInfo, Ok(abci_info_fixture))
            .map(Method::Block, Ok(block_fixture));
        let (client, driver) = MockClient::new(matcher);
        let driver_hdl = tokio::spawn(async move { driver.run().await });

        let abci_info = client.abci_info().await.unwrap();
        assert_eq!("{\"size\":0}".to_string(), abci_info.data);

        let block = client.block(Height::from(10_u32)).await.unwrap().block;
        assert_eq!(Height::from(10_u32), block.header.height);
        assert_eq!("dockerchain".parse::<Id>().unwrap(), block.header.chain_id);

        client.close();
        driver_hdl.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn mock_subscription_client() {
        let (client, driver) = MockClient::new(MockRequestMethodMatcher::default());
        let driver_hdl = tokio::spawn(async move { driver.run().await });

        let event1 = read_event("subscribe_newblock_0").await;
        let event2 = read_event("subscribe_newblock_1").await;
        let event3 = read_event("subscribe_newblock_2").await;
        let events = vec![event1, event2, event3];

        let subs1 = client.subscribe(EventType::NewBlock.into()).await.unwrap();
        let subs2 = client.subscribe(EventType::NewBlock.into()).await.unwrap();
        assert_ne!(subs1.id().to_string(), subs2.id().to_string());

        // We can do this because the underlying channels can buffer the
        // messages as we publish them.
        let subs1_events = subs1.take(3);
        let subs2_events = subs2.take(3);
        for ev in &events {
            client.publish(ev);
        }

        // Here each subscription's channel is drained.
        let subs1_events = subs1_events.collect::<Vec<Result<Event, Error>>>().await;
        let subs2_events = subs2_events.collect::<Vec<Result<Event, Error>>>().await;

        assert_eq!(3, subs1_events.len());
        assert_eq!(3, subs2_events.len());

        for i in 0..3 {
            assert!(events[i].eq(subs1_events[i].as_ref().unwrap()));
        }

        client.close();
        driver_hdl.await.unwrap().unwrap();
    }
}
