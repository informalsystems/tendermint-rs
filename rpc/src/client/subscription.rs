//! Subscription- and subscription management-related functionality.

use futures::{
    task::{Context, Poll},
    Stream,
};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{stream::StreamExt, sync::mpsc, task::JoinHandle};

use crate::{client::transport::EventConnection, event::Event, Error};

/// The subscription manager is an interface to the subscription
/// router, which runs asynchronously in a separate process.
#[derive(Debug)]
pub struct SubscriptionManager {
    router: JoinHandle<Result<(), Error>>,
    cmd_tx: mpsc::Sender<RouterCmd>,
    next_subs_id: AtomicUsize,
}

/// An interface that can be used to asynchronously receive events for a
/// particular subscription.
#[derive(Debug)]
pub struct Subscription {
    id: SubscriptionId,
    query: String,
    event_rx: mpsc::Receiver<Event>,
}

// The subscription router does the heavy lifting of managing subscriptions and
// routing incoming events to their relevant subscribers.
struct SubscriptionRouter {
    conn: EventConnection,
    cmd_rx: mpsc::Receiver<RouterCmd>,
    // Maps queries -> (maps of subscription IDs -> channels)
    subscriptions: HashMap<String, HashMap<SubscriptionId, mpsc::Sender<Event>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(usize);

#[derive(Debug)]
enum RouterCmd {
    Subscribe {
        id: SubscriptionId,
        query: String,
        event_tx: mpsc::Sender<Event>,
    },
    Unsubscribe(Subscription),
    Terminate,
}

impl SubscriptionManager {
    pub(crate) fn new(conn: EventConnection, cmd_buf_size: usize) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(cmd_buf_size);
        let router = SubscriptionRouter::new(conn, cmd_rx);
        Self {
            router: tokio::spawn(async move { router.run().await }),
            cmd_tx,
            next_subs_id: AtomicUsize::new(0),
        }
    }

    pub async fn subscribe(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription, Error> {
        let (event_tx, event_rx) = mpsc::channel(buf_size);
        let id = self.next_subs_id();
        let _ = self
            .cmd_tx
            .send(RouterCmd::Subscribe {
                id: id.clone(),
                query: query.clone(),
                event_tx,
            })
            .await
            .map_err(|e| {
                Error::internal_error(format!(
                    "failed to transmit subscription request to async task: {}",
                    e
                ))
            })?;
        Ok(Subscription {
            id,
            query,
            event_rx,
        })
    }

    pub async fn unsubscribe(&mut self, subs: Subscription) -> Result<(), Error> {
        self.cmd_tx
            .send(RouterCmd::Unsubscribe(subs))
            .await
            .map_err(|e| {
                Error::internal_error(format!(
                    "failed to transmit unsubscribe request to async task: {}",
                    e
                ))
            })
    }

    /// Gracefully terminate the subscription manager and its router (which
    /// runs in an asynchronous task).
    pub async fn terminate(mut self) -> Result<(), Error> {
        let _ = self.cmd_tx.send(RouterCmd::Terminate).await.map_err(|e| {
            Error::internal_error(format!(
                "failed to transmit termination request to async task: {}",
                e
            ))
        })?;
        self.router
            .await
            .map_err(|e| Error::internal_error(format!("failed to terminate async task: {}", e)))?
    }

    fn next_subs_id(&self) -> SubscriptionId {
        SubscriptionId(self.next_subs_id.fetch_add(1, Ordering::SeqCst))
    }
}

impl SubscriptionRouter {
    fn new(conn: EventConnection, cmd_rx: mpsc::Receiver<RouterCmd>) -> Self {
        Self {
            conn,
            cmd_rx,
            subscriptions: HashMap::new(),
        }
    }

    async fn run(mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
                Some(ev) = self.conn.event_producer.next() => self.route_event(ev).await,
                Some(cmd) = self.cmd_rx.next() => match cmd {
                    RouterCmd::Subscribe { id, query, event_tx } => self.subscribe(id, query, event_tx).await?,
                    RouterCmd::Unsubscribe(subs) => self.unsubscribe(subs).await?,
                    RouterCmd::Terminate => return self.terminate().await,
                },
            }
        }
    }

    async fn route_event(&mut self, ev: Event) {
        let subs_for_query = match self.subscriptions.get_mut(&ev.query) {
            Some(s) => s,
            None => return,
        };
        let mut disconnected = Vec::new();
        for (subs_id, tx) in subs_for_query {
            // TODO(thane): Right now we automatically remove any disconnected or full
            //       channels. We must handle full channels differently to
            //       disconnected ones.
            if let Err(_) = tx.send(ev.clone()).await {
                disconnected.push(subs_id.clone());
            }
        }
        let subs_for_query = self.subscriptions.get_mut(&ev.query).unwrap();
        for subs_id in disconnected {
            subs_for_query.remove(&subs_id);
        }
    }

    async fn subscribe(
        &mut self,
        id: SubscriptionId,
        query: String,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<(), Error> {
        let subs_for_query = match self.subscriptions.get_mut(&query) {
            Some(s) => s,
            None => {
                self.subscriptions.insert(query.clone(), HashMap::new());
                self.subscriptions.get_mut(&query).unwrap()
            }
        };
        subs_for_query.insert(id, event_tx);
        self.conn.transport.subscribe(query).await
    }

    async fn unsubscribe(&mut self, subs: Subscription) -> Result<(), Error> {
        let subs_for_query = match self.subscriptions.get_mut(&subs.query) {
            Some(s) => s,
            None => return Ok(()),
        };
        subs_for_query.remove(&subs.id);
        self.conn.transport.unsubscribe(subs.query).await
    }

    async fn terminate(mut self) -> Result<(), Error> {
        self.conn.terminate().await
    }
}

impl Stream for Subscription {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.event_rx.poll_recv(cx)
    }
}
