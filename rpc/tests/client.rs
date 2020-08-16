//! Tendermint RPC client tests.

use async_trait::async_trait;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use tendermint::block::Height;
use tendermint_rpc::event::{Event, EventData, WrappedEvent};
use tendermint_rpc::{
    Error, EventTx, FullClient, Method, MinimalClient, Request, Response, Result, Subscription,
    SubscriptionId, SubscriptionRouter,
};
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

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

#[derive(Debug)]
struct MockClient {
    responses: HashMap<Method, String>,
    driver_handle: JoinHandle<Result<()>>,
    event_tx: mpsc::Sender<Event>,
    cmd_tx: mpsc::Sender<MockClientCmd>,
}

#[async_trait]
impl MinimalClient for MockClient {
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request,
    {
        self.responses
            .get(&request.method())
            .and_then(|res| Some(R::Response::from_string(res).unwrap()))
            .ok_or_else(|| {
                Error::http_error(format!(
                    "no response mapping for request method: {}",
                    request.method()
                ))
            })
    }

    async fn close(mut self) -> Result<()> {
        self.cmd_tx.send(MockClientCmd::Close).await.unwrap();
        self.driver_handle.await.unwrap()
    }
}

#[async_trait]
impl FullClient for MockClient {
    async fn subscribe_with_buf_size(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription> {
        let (event_tx, event_rx) = mpsc::channel(buf_size);
        let (response_tx, response_rx) = oneshot::channel();
        let id = SubscriptionId::default();
        self.cmd_tx
            .send(MockClientCmd::Subscribe {
                id: id.clone(),
                query: query.clone(),
                event_tx,
                response_tx,
            })
            .await
            .unwrap();
        // We need to wait until the subscription's been created, otherwise we
        // risk introducing nondeterminism into the tests.
        response_rx.await.unwrap().unwrap();
        Ok(Subscription::new(id, query, event_rx))
    }

    async fn unsubscribe(&mut self, subscription: Subscription) -> Result<()> {
        Ok(self
            .cmd_tx
            .send(MockClientCmd::Unsubscribe(subscription))
            .await
            .unwrap())
    }
}

impl MockClient {
    fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel(10);
        let (cmd_tx, cmd_rx) = mpsc::channel(10);
        let driver = MockClientDriver::new(event_rx, cmd_rx);
        let driver_hdl = tokio::spawn(async move { driver.run().await });
        Self {
            responses: HashMap::new(),
            driver_handle: driver_hdl,
            event_tx,
            cmd_tx,
        }
    }

    fn map(mut self, method: Method, response: String) -> Self {
        self.responses.insert(method, response);
        self
    }

    async fn publish(&mut self, ev: Event) {
        match &ev.data {
            EventData::NewBlock { block, .. } => println!(
                "Sending NewBlock event for height {}",
                block.as_ref().unwrap().header.height
            ),
            _ => (),
        }
        self.event_tx.send(ev).await.unwrap();
    }
}

#[derive(Debug)]
enum MockClientCmd {
    Subscribe {
        id: SubscriptionId,
        query: String,
        event_tx: EventTx,
        response_tx: oneshot::Sender<Result<()>>,
    },
    Unsubscribe(Subscription),
    Close,
}

#[derive(Debug)]
struct MockClientDriver {
    event_rx: mpsc::Receiver<Event>,
    cmd_rx: mpsc::Receiver<MockClientCmd>,
    router: SubscriptionRouter,
}

impl MockClientDriver {
    // `event_rx` simulates an incoming event stream (e.g. by way of the
    // WebSocket connection).
    fn new(event_rx: mpsc::Receiver<Event>, cmd_rx: mpsc::Receiver<MockClientCmd>) -> Self {
        Self {
            event_rx,
            cmd_rx,
            router: SubscriptionRouter::default(),
        }
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(ev) = self.event_rx.next() => {
                    match &ev.data {
                        EventData::NewBlock { block, .. } => println!(
                            "Publishing NewBlock event at height {}",
                            block.as_ref().unwrap().header.height,
                        ),
                        _ => (),
                    }
                    self.router.publish(ev).await;
                    ()
                }
                Some(cmd) = self.cmd_rx.next() => match cmd {
                    MockClientCmd::Subscribe { id, query, event_tx, response_tx } => {
                        self.router.add(id, query, event_tx);
                        response_tx.send(Ok(())).unwrap();
                        ()
                    },
                    MockClientCmd::Unsubscribe(subs) => {
                        self.router.remove(subs);
                        ()
                    },
                    MockClientCmd::Close => return Ok(()),
                }
            }
        }
    }
}

#[tokio::test]
async fn minimal_client() {
    let client = MockClient::new()
        .map(Method::AbciInfo, read_json_fixture("abci_info").await)
        .map(Method::Block, read_json_fixture("block").await);

    let abci_info = client.abci_info().await.unwrap();
    assert_eq!("GaiaApp".to_string(), abci_info.data);

    let block = client.block(10).await.unwrap().block;
    assert_eq!(Height::from(10), block.header.height);
    assert_eq!("cosmoshub-2", block.header.chain_id.as_str());

    client.close().await.unwrap();
}

#[tokio::test]
async fn full_client() {
    let mut client = MockClient::new();
    let incoming_events = vec![
        read_event("event_new_block_1").await,
        read_event("event_new_block_2").await,
        read_event("event_new_block_3").await,
    ];
    let expected_heights = vec![Height::from(165), Height::from(166), Height::from(167)];

    let subs1 = client
        .subscribe("tm.event='NewBlock'".to_string())
        .await
        .unwrap();
    let subs2 = client
        .subscribe("tm.event='NewBlock'".to_string())
        .await
        .unwrap();

    let subs1_events_task =
        tokio::spawn(async move { subs1.take(3).collect::<Vec<Result<Event>>>().await });
    let subs2_events_task =
        tokio::spawn(async move { subs2.take(3).collect::<Vec<Result<Event>>>().await });

    println!("Publishing incoming events...");
    for ev in incoming_events {
        client.publish(ev).await;
    }

    println!("Collecting incoming events...");
    let subs1_events = subs1_events_task.await.unwrap();
    let subs2_events = subs2_events_task.await.unwrap();

    client.close().await.unwrap();

    assert_eq!(3, subs1_events.len());
    assert_eq!(3, subs2_events.len());

    println!("Checking collected events...");
    for i in 0..3 {
        let subs1_event = subs1_events[i].as_ref().unwrap();
        let subs2_event = subs2_events[i].as_ref().unwrap();
        match &subs1_event.data {
            EventData::NewBlock { block, .. } => {
                assert_eq!(expected_heights[i], block.as_ref().unwrap().header.height);
            }
            _ => panic!("invalid event type for subs1: {:?}", subs1_event),
        }
        match &subs2_event.data {
            EventData::NewBlock { block, .. } => {
                assert_eq!(expected_heights[i], block.as_ref().unwrap().header.height);
            }
            _ => panic!("invalid event type for subs2: {:?}", subs2_event),
        }
    }
}
