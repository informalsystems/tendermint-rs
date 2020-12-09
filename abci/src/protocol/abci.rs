//! ABCI low-level protocol handler.

use crate::result::{Result, ResultError};
use crate::Error;
use futures::ready;
use futures::task::{Context, Poll};
use futures::{Sink, Stream};
use pin_project::pin_project;
use std::pin::Pin;
use tendermint_proto::abci;
use tendermint_proto::prost::bytes::{Bytes, BytesMut};
use tendermint_proto::prost::Message;

#[pin_project]
pub struct AbciStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    #[pin]
    inner: S,
}

impl<S> AbciStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    /// Constructor.
    pub fn new(source_stream: S) -> Self {
        Self {
            inner: source_stream,
        }
    }
}

impl<S> Stream for AbciStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    type Item = Result<abci::Response>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let inner: Pin<&mut S> = self.project().inner;

        if let Some(result) = ready!(inner.poll_next(cx)) {
            let bytes = match result {
                Ok(b) => b,
                Err(e) => return Poll::Ready(Some(Err(e))),
            };
            // Try to decode the incoming bytes as a response
            let result: Result<abci::Response> = abci::Response::decode(bytes).map_err(Into::into);
            Poll::Ready(Some(result))
        } else {
            // The underlying stream terminated
            Poll::Ready(None)
        }
    }
}

impl<S> Sink<abci::Request> for AbciStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    type Error = ResultError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: abci::Request) -> Result<()> {
        let mut buf = BytesMut::new();
        item.encode(&mut buf).map_err(Error::ProtobufEncode)?;
        self.project().inner.start_send(buf.freeze())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_close(cx)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::protocol::tsp::test::BytesStream;
    use crate::TspStream;
    use futures::SinkExt;

    #[test]
    fn abci_decode() {
        // A length-delimited echo response
        let bytes_stream = BytesStream::new(vec![Ok(Bytes::from(vec![
            30, 18, 13, 10, 11, 72, 101, 108, 108, 111, 32, 65, 66, 67, 73, 33,
        ]))]);
        let abci_stream = AbciStream::new(TspStream::new(bytes_stream));
        let parsed_messages = futures::executor::block_on_stream(abci_stream)
            .into_iter()
            .collect::<Vec<Result<abci::Response>>>();
        assert_eq!(parsed_messages.len(), 1);
        let msg = parsed_messages[0].as_ref().unwrap().value.as_ref().unwrap();
        if let abci::response::Value::Echo(res) = msg {
            assert_eq!(res.message, "Hello ABCI!");
        } else {
            panic!("Unexpected message type: {:?}", msg);
        }
    }

    #[test]
    fn abci_encode() {
        let request = abci::Request {
            value: Some(abci::request::Value::Echo(abci::RequestEcho {
                message: "Hello ABCI!".to_string(),
            })),
        };
        let bytes_stream = BytesStream::new(vec![]);
        let write_buf_mtx = bytes_stream.write_buf_ref();
        let mut abci_stream = AbciStream::new(TspStream::new(bytes_stream));
        futures::executor::block_on(abci_stream.send(request)).unwrap();
        let write_buf = write_buf_mtx.lock().unwrap();
        assert_eq!(
            write_buf.to_vec(),
            vec![30, 10, 13, 10, 11, 72, 101, 108, 108, 111, 32, 65, 66, 67, 73, 33]
        );
    }
}
