//! Synchronization primitives specific to the Tendermint RPC client.
//!
//! At present, this wraps Tokio's synchronization primitives and provides some
//! convenience methods. We also only implement unbounded channels at present.
//! In future, if RPC consumers need it, we will implement bounded channels.

use crate::{Error, Result};
use futures::task::{Context, Poll};
use tokio::sync::mpsc;

/// Constructor for an unbounded channel.
pub fn unbounded<T>() -> (ChannelTx<T>, ChannelRx<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (ChannelTx(tx), ChannelRx(rx))
}

/// Sender interface for a channel.
///
/// Can be cloned because the underlying channel used is
/// [`mpsc`](https://docs.rs/tokio/*/tokio/sync/mpsc/index.html).
#[derive(Debug, Clone)]
pub struct ChannelTx<T>(mpsc::UnboundedSender<T>);

impl<T> ChannelTx<T> {
    pub async fn send(&mut self, value: T) -> Result<()> {
        self.0.send(value).map_err(|e| {
            Error::client_internal_error(format!(
                "failed to send message to internal channel: {}",
                e
            ))
        })
    }
}

/// Receiver interface for a channel.
#[derive(Debug)]
pub struct ChannelRx<T>(mpsc::UnboundedReceiver<T>);

impl<T> ChannelRx<T> {
    /// Wait indefinitely until we receive a value from the channel (or the
    /// channel is closed).
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.0.poll_recv(cx)
    }
}
