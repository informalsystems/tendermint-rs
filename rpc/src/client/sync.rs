//! Synchronization primitives specific to the Tendermint RPC client.
//!
//! At present, this wraps Tokio's synchronization primitives and provides some
//! convenience methods. We also only implement unbounded channels at present.
//! In future, if RPC consumers need it, we will implement bounded channels.

use core::pin::Pin;

use futures::task::{Context, Poll};
use futures::Stream;
use pin_project::pin_project;
use tokio::sync::mpsc;

use crate::Error;

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
    pub fn send(&self, value: T) -> Result<(), Error> {
        self.0.send(value).map_err(Error::send)
    }
}

/// Receiver interface for a channel.
#[pin_project]
#[derive(Debug)]
pub struct ChannelRx<T>(#[pin] mpsc::UnboundedReceiver<T>);

impl<T> ChannelRx<T> {
    /// Wait indefinitely until we receive a value from the channel (or the
    /// channel is closed).
    #[allow(dead_code)]
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }
}

impl<T> Stream for ChannelRx<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().0.poll_recv(cx)
    }
}
