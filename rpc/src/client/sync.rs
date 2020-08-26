//! Synchronization primitives specific to the Tendermint RPC client.
//!
//! At present, this wraps Tokio's synchronization primitives and provides some
//! conveniences, such as an interface to a channel without caring about
//! whether it's bounded or unbounded.

use crate::{Error, Result};
use futures::task::{Context, Poll};
use tokio::sync::mpsc;

/// Constructor for a bounded channel with maximum capacity of `buf_size`
/// elements.
pub fn bounded<T>(buf_size: usize) -> (ChannelTx<T>, ChannelRx<T>) {
    let (tx, rx) = mpsc::channel(buf_size);
    (ChannelTx::Bounded(tx), ChannelRx::Bounded(rx))
}

/// Constructor for an unbounded channel.
pub fn unbounded<T>() -> (ChannelTx<T>, ChannelRx<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (ChannelTx::Unbounded(tx), ChannelRx::Unbounded(rx))
}

/// Generic sender interface on bounded and unbounded channels for
/// `Result<Event>` instances.
///
/// Can be cloned because the underlying channel used is
/// [`mpsc`](https://docs.rs/tokio/*/tokio/sync/mpsc/index.html).
#[derive(Debug, Clone)]
pub enum ChannelTx<T> {
    Bounded(mpsc::Sender<T>),
    Unbounded(mpsc::UnboundedSender<T>),
}

impl<T> ChannelTx<T> {
    pub async fn send(&mut self, value: T) -> Result<()> {
        match self {
            ChannelTx::Bounded(ref mut tx) => tx.send(value).await,
            ChannelTx::Unbounded(ref mut tx) => tx.send(value),
        }
        .map_err(|e| {
            Error::client_internal_error(format!(
                "failed to send message to internal channel: {}",
                e
            ))
        })
    }
}

/// Generic receiver interface on bounded and unbounded channels.
#[derive(Debug)]
pub enum ChannelRx<T> {
    /// A channel that can contain up to a fixed number of items.
    Bounded(mpsc::Receiver<T>),
    /// A channel that is unconstrained (except by system resources, of
    /// course).
    Unbounded(mpsc::UnboundedReceiver<T>),
}

impl<T> ChannelRx<T> {
    pub async fn recv(&mut self) -> Option<T> {
        match self {
            ChannelRx::Bounded(ref mut rx) => rx.recv().await,
            ChannelRx::Unbounded(ref mut rx) => rx.recv().await,
        }
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        match self {
            ChannelRx::Bounded(ref mut rx) => rx.poll_recv(cx),
            ChannelRx::Unbounded(ref mut rx) => rx.poll_recv(cx),
        }
    }
}
