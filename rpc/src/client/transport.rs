//! Transport layer abstraction for the Tendermint RPC client.

use async_trait::async_trait;
use futures::{
    task::{Context, Poll},
    Stream,
};
use std::pin::Pin;
use tokio::sync::mpsc;

use crate::{events::Event, Error};

pub mod http_ws;

/// Transport layer abstraction for interacting with real or mocked Tendermint
/// full nodes.
#[async_trait]
pub trait Transport: std::fmt::Debug {
    /// Perform a request to the remote endpoint, expecting a response.
    async fn request(&self, request: String) -> Result<String, Error>;

    /// Provides access to a stream of incoming events. These would be
    /// produced, for example, once at least one subscription has been
    /// initiated by the RPC client.
    async fn new_event_connection(&self, event_buf_size: usize) -> Result<EventConnection, Error>;
}

/// The part of the transport layer that exclusively deals with
/// subscribe/unsubscribe requests.
#[async_trait]
pub trait SubscriptionTransport: std::fmt::Debug + Send {
    /// Send a subscription request through the transport layer.
    async fn subscribe(&mut self, query: String) -> Result<(), Error>;

    /// Send an unsubscribe request through the transport layer.
    async fn unsubscribe(&mut self, query: String) -> Result<(), Error>;

    /// Attempt to gracefully terminate the transport layer.
    async fn close(&mut self) -> Result<(), Error>;
}

/// An `EventConnection` allows us to send subscribe/unsubscribe requests via
/// the transport layer, as well as receive incoming events from subscriptions.
#[derive(Debug)]
pub struct EventConnection {
    // The `EventConnection` struct is a workaround for the fact that we need
    // to use `Transport` as a trait object, but trait objects are not allowed
    // to use generics in their method signatures.
    pub transport: Box<dyn SubscriptionTransport>,
    pub event_producer: EventProducer,
}

impl EventConnection {
    pub fn new(transport: Box<dyn SubscriptionTransport>, event_producer: EventProducer) -> Self {
        Self {
            transport,
            event_producer,
        }
    }

    pub async fn terminate(&mut self) -> Result<(), Error> {
        self.transport.close().await
    }
}

#[derive(Debug)]
pub struct EventProducer {
    event_rx: mpsc::Receiver<Event>,
}

impl Stream for EventProducer {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.event_rx.poll_recv(cx)
    }
}

impl EventProducer {
    pub fn new(event_rx: mpsc::Receiver<Event>) -> Self {
        Self { event_rx }
    }
}
