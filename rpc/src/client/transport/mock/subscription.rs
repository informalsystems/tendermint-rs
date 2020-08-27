//! Subscription functionality for the Tendermint RPC mock client.

use crate::client::subscription::TerminateSubscription;
use crate::client::sync::{unbounded, ChannelRx, ChannelTx};
use crate::client::{ClosableClient, SubscriptionRouter};
use crate::event::Event;
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
    cmd_tx: ChannelTx<DriverCmd>,
    terminate_tx: ChannelTx<TerminateSubscription>,
}

#[async_trait]
impl SubscriptionClient for MockSubscriptionClient {
    async fn subscribe(&mut self, query: String) -> Result<Subscription> {
        let (event_tx, event_rx) = unbounded();
        let (result_tx, mut result_rx) = unbounded();
        let id = SubscriptionId::default();
        self.send_cmd(DriverCmd::Subscribe {
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

        Ok(Subscription::new(
            id,
            query,
            event_rx,
            self.terminate_tx.clone(),
        ))
    }
}

#[async_trait]
impl ClosableClient for MockSubscriptionClient {
    async fn close(mut self) -> Result<()> {
        self.send_cmd(DriverCmd::Close).await?;
        self.driver_hdl.await.map_err(|e| {
            Error::client_internal_error(format!(
                "failed to terminate mock client driver task: {}",
                e
            ))
        })?
    }
}

impl MockSubscriptionClient {
    /// Publish the given event to all subscribers whose queries match that of
    /// the event.
    pub async fn publish(&mut self, ev: Event) -> Result<()> {
        self.event_tx.send(ev).await
    }

    async fn send_cmd(&mut self, cmd: DriverCmd) -> Result<()> {
        self.cmd_tx.send(cmd).await
    }
}

impl Default for MockSubscriptionClient {
    fn default() -> Self {
        let (event_tx, event_rx) = unbounded();
        let (cmd_tx, cmd_rx) = unbounded();
        let (terminate_tx, terminate_rx) = unbounded();
        let driver = MockSubscriptionClientDriver::new(event_rx, cmd_rx, terminate_rx);
        let driver_hdl = tokio::spawn(async move { driver.run().await });
        Self {
            driver_hdl,
            event_tx,
            cmd_tx,
            terminate_tx,
        }
    }
}

#[derive(Debug)]
struct MockSubscriptionClientDriver {
    event_rx: ChannelRx<Event>,
    cmd_rx: ChannelRx<DriverCmd>,
    terminate_rx: ChannelRx<TerminateSubscription>,
    router: SubscriptionRouter,
}

impl MockSubscriptionClientDriver {
    fn new(
        event_rx: ChannelRx<Event>,
        cmd_rx: ChannelRx<DriverCmd>,
        terminate_rx: ChannelRx<TerminateSubscription>,
    ) -> Self {
        Self {
            event_rx,
            cmd_rx,
            terminate_rx,
            router: SubscriptionRouter::default(),
        }
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(ev) = self.event_rx.recv() => self.router.publish(ev).await,
                Some(cmd) = self.cmd_rx.recv() => match cmd {
                    DriverCmd::Subscribe {
                        id,
                        query,
                        event_tx,
                        result_tx,
                    } => self.subscribe(id, query, event_tx, result_tx).await?,
                    DriverCmd::Close => return Ok(()),
                },
                Some(subs_term) = self.terminate_rx.recv() => self.unsubscribe(subs_term).await?,
            }
        }
    }

    async fn subscribe(
        &mut self,
        id: SubscriptionId,
        query: String,
        event_tx: ChannelTx<Result<Event>>,
        mut result_tx: ChannelTx<Result<()>>,
    ) -> Result<()> {
        self.router.add(&id, query, event_tx);
        result_tx.send(Ok(())).await
    }

    async fn unsubscribe(&mut self, mut subs_term: TerminateSubscription) -> Result<()> {
        self.router.remove(&subs_term.id, subs_term.query.clone());
        subs_term.result_tx.send(Ok(())).await
    }
}

#[derive(Debug)]
pub enum DriverCmd {
    Subscribe {
        id: SubscriptionId,
        query: String,
        event_tx: ChannelTx<Result<Event>>,
        result_tx: ChannelTx<Result<()>>,
    },
    Close,
}

#[cfg(test)]
mod test {
    use super::*;
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
        let events = vec![
            read_event("event_new_block_1").await,
            read_event("event_new_block_2").await,
            read_event("event_new_block_3").await,
        ];

        let subs1 = client
            .subscribe("tm.event='NewBlock'".to_string())
            .await
            .unwrap();
        let subs2 = client
            .subscribe("tm.event='NewBlock'".to_string())
            .await
            .unwrap();
        assert_ne!(subs1.id, subs2.id);

        let subs1_events = take_from_subs_and_terminate(subs1, 3);
        let subs2_events = take_from_subs_and_terminate(subs2, 3);
        for ev in &events {
            client.publish(ev.clone()).await.unwrap();
        }

        let (subs1_events, subs2_events) =
            (subs1_events.await.unwrap(), subs2_events.await.unwrap());

        assert_eq!(3, subs1_events.len());
        assert_eq!(3, subs2_events.len());

        for i in 0..3 {
            assert!(events[i].eq(subs1_events[i].as_ref().unwrap()));
        }

        client.close().await.unwrap();
    }
}
