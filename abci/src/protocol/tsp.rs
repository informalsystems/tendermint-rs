//! Tendermint socket protocol.
//!
//! See https://docs.tendermint.com/master/spec/abci/client-server.html#tsp

use crate::result::ResultError;
use crate::Result;
use futures::task::{Context, Poll};
use futures::{Sink, Stream};
use pin_project::pin_project;
use std::pin::Pin;
use tendermint_proto::prost::bytes::{Buf, BufMut, Bytes, BytesMut};
//use tendermint_proto::prost::encoding::{decode_varint, encode_varint};

// We expect a varint to be at most 9 bytes long, since it can maximally
// represent a `u64`.
const MAX_VARINT_LEN: usize = 9;

/// Tendermint socket protocol stream (readable and writable).
///
/// The stream parses incoming unstructured chunks of bytes into
/// length-delimited chunks of bytes, which can then be interpreted as Protobuf
/// messages.
///
/// The writer assumes that each outgoing item needs a length prefix, and
/// automatically prepends this length prefix.
///
/// See the [Tendermint Socket Protocol][tsp] for more details.
///
/// [tsp]: https://docs.tendermint.com/master/spec/abci/client-server.html#tsp
#[pin_project]
#[derive(Debug)]
pub struct TspStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    #[pin]
    inner: S,

    // Our buffer of data from the underlying stream.
    buf: BytesMut,
}

impl<S> TspStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    /// Constructor.
    pub fn new(source: S) -> Self {
        Self {
            inner: source,
            buf: BytesMut::new(),
        }
    }
}

impl<S> Stream for TspStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let buf: &mut BytesMut = this.buf;

        if let Some(next) = try_extract_next_msg(buf) {
            return Poll::Ready(Some(next));
        }

        while let Poll::Ready(opt) = this.inner.as_mut().poll_next(cx) {
            match opt {
                Some(res) => match res {
                    Ok(b) => buf.extend_from_slice(&b),
                    Err(e) => return Poll::Ready(Some(Err(e))),
                },
                // The inner stream terminated
                None => return Poll::Ready(None),
            }

            if let Some(next) = try_extract_next_msg(buf) {
                return Poll::Ready(Some(next));
            }
        }

        Poll::Pending
    }
}

fn try_extract_next_msg(b: &mut BytesMut) -> Option<Result<Bytes>> {
    if b.is_empty() {
        return None;
    }
    // Get a reference to the original buffer
    let mut b_clone = b.clone();
    let expected_len = match decode_varint(&mut b_clone) {
        Ok(len) => len as usize,
        Err(e) => {
            return if b.len() >= MAX_VARINT_LEN {
                // We should have been able to decode a varint from a
                // buffer of this size
                Some(Err(e))
            } else {
                // We're probably waiting for more data
                None
            };
        }
    };
    // If we now have enough data to extract a message
    if b_clone.len() >= expected_len {
        // Extract the next message
        let next = b_clone.split_to(expected_len).freeze();
        // We can now safely advance our original buffer
        b.advance(b.len() - b_clone.remaining());
        Some(Ok(next))
    } else {
        None
    }
}

impl<S> Sink<Bytes> for TspStream<S>
where
    S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError>,
{
    type Error = ResultError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<()> {
        let mut length_prefixed = BytesMut::new();
        encode_varint(item.len() as u64, &mut length_prefixed);
        length_prefixed.put(item);
        self.project().inner.start_send(length_prefixed.freeze())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_close(cx)
    }
}

// The Go version of Tendermint uses https://golang.org/pkg/encoding/binary/#PutVarint,
// which is the signed variant of https://golang.org/pkg/encoding/binary/#PutUvarint
// (the latter being the equivalent of Prost's `encode_varint` method).
//
// The signed variant of this function first shifts the value one bit to the
// left before encoding it, and then shifts it back upon decoding.
fn encode_varint<B>(value: u64, mut buf: &mut B)
where
    B: BufMut,
{
    tendermint_proto::prost::encoding::encode_varint(value << 1, &mut buf)
}

fn decode_varint<B>(mut buf: &mut B) -> Result<u64>
where
    B: Buf,
{
    tendermint_proto::prost::encoding::decode_varint(&mut buf)
        .map(|i| i >> 1)
        .map_err(|e| e.into())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use eyre::eyre;
    use futures::SinkExt;
    use std::sync::{Arc, Mutex};

    #[pin_project]
    pub struct BytesStream {
        read_buf: Vec<Result<Bytes>>,
        write_buf: Arc<Mutex<BytesMut>>,
    }

    impl BytesStream {
        pub fn new(read_buf: Vec<Result<Bytes>>) -> Self {
            Self {
                read_buf,
                write_buf: Arc::new(Mutex::new(BytesMut::new())),
            }
        }

        pub fn write_buf_ref(&self) -> Arc<Mutex<BytesMut>> {
            self.write_buf.clone()
        }
    }

    impl Stream for BytesStream {
        type Item = Result<Bytes>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let read_buf: &mut Vec<Result<Bytes>> = self.project().read_buf;
            if read_buf.is_empty() {
                return Poll::Ready(None);
            }
            Poll::Ready(Some(read_buf.remove(0)))
        }
    }

    impl Sink<Bytes> for BytesStream {
        type Error = ResultError;

        fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<()> {
            let write_buf_mtx: &mut Arc<Mutex<BytesMut>> = self.project().write_buf;
            let mut write_buf = write_buf_mtx.lock().unwrap();
            write_buf.put(item);
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[test]
    fn tsp_reader() {
        let msgs = vec![
            "Hello",
            "Tendermint",
            "this",
            "is",
            "length-delimited",
            "messaging",
        ];
        // A buffer containing all of the above messages, each with a
        // varint-encoded length delimiter prefix
        let msgs_with_prefixes = build_msg_buffer(&msgs);

        // Try splitting the buffer by different lengths
        for n_chunks in 1..msgs_with_prefixes.len() {
            // Build our TSP reader
            let mut msgs_with_prefixes_clone = msgs_with_prefixes.clone();
            let results = split_buffer_into_results(&mut msgs_with_prefixes_clone, n_chunks);
            let stream = BytesStream::new(results);
            let reader = TspStream::new(stream);

            // Consume all the messages from the TSP reader's stream
            let parsed_messages = collect_tsp_reader(reader);
            assert_eq!(parsed_messages.len(), msgs.len());
            for i in 0..msgs.len() {
                assert_eq!(parsed_messages[i].as_ref().unwrap(), msgs[i]);
            }
        }
    }

    #[test]
    fn tsp_reader_underlying_stream_failure() {
        let broken_results: Vec<Result<Bytes>> = vec![Err(eyre!("failed"))];
        let stream = BytesStream::new(broken_results);
        let reader = TspStream::new(stream);
        let parsed_messages = collect_tsp_reader(reader);
        assert_eq!(parsed_messages.len(), 1);
        assert!(parsed_messages[0].is_err());
    }

    #[test]
    fn tsp_read_write() {
        let msgs = vec![
            "Hello",
            "Tendermint",
            "this",
            "is",
            "length-delimited",
            "messaging",
        ];
        let mut msg_stream = futures::stream::iter(
            msgs.clone()
                .into_iter()
                .map(Bytes::from)
                .map(Result::Ok)
                .collect::<Vec<Result<Bytes>>>(),
        );
        let bytes_stream = BytesStream::new(vec![]);
        let write_buf_mtx = bytes_stream.write_buf_ref();
        let mut writer = TspStream::new(bytes_stream);
        futures::executor::block_on(writer.send_all(&mut msg_stream)).unwrap();
        let buf = write_buf_mtx.lock().unwrap().clone().freeze();
        drop(write_buf_mtx);
        assert!(!buf.is_empty());

        let reader = TspStream::new(BytesStream::new(vec![Ok(buf)]));
        let parsed_messages = collect_tsp_reader(reader);
        assert_eq!(parsed_messages.len(), msgs.len());
        for i in 0..msgs.len() {
            assert_eq!(parsed_messages[i].as_ref().unwrap(), msgs[i]);
        }
    }

    fn build_msg_buffer(msgs: &[&str]) -> Bytes {
        let mut msgs_with_prefixes = BytesMut::new();
        for msg in msgs {
            let mut msg_with_prefix = BytesMut::new();
            encode_varint(msg.len() as u64, &mut msg_with_prefix);
            msg_with_prefix.extend_from_slice(msg.as_bytes());
            msgs_with_prefixes.extend(msg_with_prefix);
        }
        msgs_with_prefixes.freeze()
    }

    fn split_buffer_into_results(buf: &mut Bytes, min_chunks: usize) -> Vec<Result<Bytes>> {
        let chunk_len = ((buf.len() * 10) / min_chunks) / 10;
        let mut results = Vec::new();
        for _chunk in 0..min_chunks {
            let split_len = if buf.len() < chunk_len {
                buf.len()
            } else {
                chunk_len
            };
            results.push(Ok(buf.split_to(split_len)));
        }
        if !buf.is_empty() {
            results.push(Ok(buf.split_to(buf.len())))
        }
        results
    }

    fn collect_tsp_reader<S>(reader: TspStream<S>) -> Vec<Result<String>>
    where
        S: Stream<Item = Result<Bytes>> + Sink<Bytes, Error = ResultError> + Unpin,
    {
        futures::executor::block_on_stream(reader)
            .into_iter()
            .map(|res| res.map(|b| String::from_utf8(b.to_vec()).unwrap()))
            .collect::<Vec<Result<String>>>()
    }
}
