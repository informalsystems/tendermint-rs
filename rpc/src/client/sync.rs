//! Synchronization primitives specific to the Tendermint RPC client.
//!
//! At present, this wraps Tokio's synchronization primitives and provides some
//! convenience methods. We also only implement unbounded channels at present.
//! In future, if RPC consumers need it, we will implement bounded channels.

use std::pin::Pin;

use crate::{Error, Result};
use futures::task::{Context, Poll};
use futures::Stream;
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

    /// Pinning is structural for the underlying channel.
    /// As such we can project the underlying channel as a pinned value.
    ///
    /// See https://doc.rust-lang.org/std/pin/index.html#pinning-is-structural-for-field
    fn pin_get(self: Pin<&mut Self>) -> Pin<&mut mpsc::UnboundedReceiver<T>> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }
    }
}

impl<T> Stream for ChannelRx<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.pin_get().poll_next(cx)
    }
}
