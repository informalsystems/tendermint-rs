//! Subscription functionality for the Tendermint RPC mock client.

use crate::client::subscription::{SubscriptionDriverCmd, SubscriptionRouter};
use crate::client::sync::{unbounded, ChannelRx, ChannelTx};
use crate::event::Event;
use crate::query::Query;
use crate::{Error, Result, Subscription, SubscriptionClient, SubscriptionId};
use async_trait::async_trait;
use tokio::task::JoinHandle;

/// A mock client that facilitates [`Event`] subscription.
///
/// Creating a `MockSubscriptionClient` will immediately spawn an asynchronous
/// driver task that handles routing of incoming [`Event`]s. The
/// `MockSubscriptionClient` then effectively becomes a handle to the
/// asynchronous driver.
///
/// [`Event`]: event/struct.Event.html
#[derive(Debug)]
pub struct MockSubscriptionClient {
    driver_hdl: JoinHandle<Result<()>>,
    event_tx: ChannelTx<Event>,
    cmd_tx: ChannelTx<SubscriptionDriverCmd>,
}

#[async_trait]
impl SubscriptionClient for MockSubscriptionClient {
    async fn subscribe(&mut self, query: Query) -> Result<Subscription> {
        let (event_tx, event_rx) = unbounded();
        let (result_tx, mut result_rx) = unbounded();
        let id = SubscriptionId::default();
        self.send_cmd(SubscriptionDriverCmd::Subscribe {
            id: id.clone(),
            query: query.clone(),
            event_tx,
            result_tx,
        })
        .await?;
        result_rx.recv().await.ok_or_else(|| {
            Error::client_internal_error(
                "failed to receive subscription confirmation from mock client driver",
            )
        })??;

        Ok(Subscription::new(id, query, event_rx, self.cmd_tx.clone()))
    }
}

impl MockSubscriptionClient {
    /// Publish the given event to all subscribers whose queries match that of
    /// the event.
    pub async fn publish(&mut self, ev: Event) -> Result<()> {
        self.event_tx.send(ev).await
    }

    async fn send_cmd(&mut self, cmd: SubscriptionDriverCmd) -> Result<()> {
        self.cmd_tx.send(cmd).await
    }

    /// Attempt to gracefully close this client.
    pub async fn close(mut self) -> Result<()> {
        self.send_cmd(SubscriptionDriverCmd::Terminate).await?;
        self.driver_hdl.await.map_err(|e| {
            Error::client_internal_error(format!(
                "failed to terminate mock client driver task: {}",
                e
            ))
        })?
    }
}

impl Default for MockSubscriptionClient {
    fn default() -> Self {
        let (event_tx, event_rx) = unbounded();
        let (cmd_tx, cmd_rx) = unbounded();
        let driver = MockSubscriptionClientDriver::new(event_rx, cmd_rx);
        let driver_hdl = tokio::spawn(async move { driver.run().await });
        Self {
            driver_hdl,
            event_tx,
            cmd_tx,
        }
    }
}

#[derive(Debug)]
struct MockSubscriptionClientDriver {
    event_rx: ChannelRx<Event>,
    cmd_rx: ChannelRx<SubscriptionDriverCmd>,
    router: SubscriptionRouter,
}

impl MockSubscriptionClientDriver {
    fn new(event_rx: ChannelRx<Event>, cmd_rx: ChannelRx<SubscriptionDriverCmd>) -> Self {
        Self {
            event_rx,
            cmd_rx,
            router: SubscriptionRouter::default(),
        }
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(ev) = self.event_rx.recv() => self.router.publish(ev).await,
                Some(cmd) = self.cmd_rx.recv() => match cmd {
                    SubscriptionDriverCmd::Subscribe {
                        id,
                        query,
                        event_tx,
                        result_tx,
                    } => self.subscribe(id, query, event_tx, result_tx).await?,
                    SubscriptionDriverCmd::Unsubscribe {
                        id,
                        query,
                        result_tx,
                    } => self.unsubscribe(id, query, result_tx).await?,
                    SubscriptionDriverCmd::Terminate => return Ok(()),
                },
            }
        }
    }

    async fn subscribe(
        &mut self,
        id: SubscriptionId,
        query: impl ToString,
        event_tx: ChannelTx<Result<Event>>,
        mut result_tx: ChannelTx<Result<()>>,
    ) -> Result<()> {
        self.router.add(&id, query.to_string(), event_tx);
        result_tx.send(Ok(())).await
    }

    async fn unsubscribe(
        &mut self,
        id: SubscriptionId,
        query: impl ToString,
        mut result_tx: ChannelTx<Result<()>>,
    ) -> Result<()> {
        self.router.remove(&id, query.to_string());
        result_tx.send(Ok(())).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::query::EventType;
    use crate::Response;
    use futures::StreamExt;
    use std::path::PathBuf;
    use tokio::fs;

    async fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json"))
            .await
            .unwrap()
    }

    async fn read_event(name: &str) -> Event {
        Event::from_string(&read_json_fixture(name).await).unwrap()
    }

    fn take_from_subs_and_terminate(
        mut subs: Subscription,
        count: usize,
    ) -> JoinHandle<Vec<Result<Event>>> {
        tokio::spawn(async move {
            let mut res = Vec::new();
            while let Some(res_ev) = subs.next().await {
                res.push(res_ev);
                if res.len() >= count {
                    break;
                }
            }
            subs.terminate().await.unwrap();
            res
        })
    }

    #[tokio::test]
    async fn mock_subscription_client() {
        let mut client = MockSubscriptionClient::default();
        let event1 = read_event("event_new_block_1").await;
        let event2 = read_event("event_new_block_2").await;
        let event3 = read_event("event_new_block_3").await;
        let events = vec![event1, event2, event3];

        let subs1 = client.subscribe(EventType::NewBlock.into()).await.unwrap();
        let subs2 = client.subscribe(EventType::NewBlock.into()).await.unwrap();
        assert_ne!(subs1.id, subs2.id);

        let subs1_events = take_from_subs_and_terminate(subs1, 3);
        let subs2_events = take_from_subs_and_terminate(subs2, 3);
        for ev in &events {
            client.publish(ev.clone()).await.unwrap();
        }

        let subs1_events = subs1_events.await.unwrap();
        let subs2_events = subs2_events.await.unwrap();

        assert_eq!(3, subs1_events.len());
        assert_eq!(3, subs2_events.len());

        for i in 0..3 {
            assert!(events[i].eq(subs1_events[i].as_ref().unwrap()));
        }

        client.close().await.unwrap();
    }
}
