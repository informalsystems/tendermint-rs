//! `std` runtime-specific types.

use crate::codec::{TspDecoder, TspEncoder};
use crate::runtime::{
    ChannelNotify, ClientCodec, Receiver, Runtime, Sender, ServerCodec, TaskSpawner, TcpListener,
    TcpStream,
};
use crate::{Error, Result};
use bytes::{Buf, BytesMut};
use std::io::{Read, Write};
use std::net::SocketAddr;
use tendermint::abci::request::Request;
use tendermint::abci::response::Response;
use tendermint::abci::{request, response};

const CODEC_READ_BUF_SIZE: usize = 128;

/// The built-in Rust standard library is our runtime.
pub struct Std;

impl Runtime for Std {
    type TcpStream = StdTcpStream;
    type TcpListener = StdTcpListener;
    type TaskSpawner = StdTaskSpawner;
    type ServerCodec = StdServerCodec;
    type ClientCodec = StdClientCodec;
    type ChannelNotify = StdChannelNotify;
}

pub struct StdTcpStream(std::net::TcpStream);

impl TcpStream for StdTcpStream {
    fn connect(addr: &str) -> Result<Self> {
        Ok(Self(std::net::TcpStream::connect(addr)?))
    }
}

pub struct StdTcpListener(std::net::TcpListener);

impl TcpListener<StdTcpStream> for StdTcpListener {
    fn bind(addr: &str) -> Result<Self> {
        Ok(Self(std::net::TcpListener::bind(addr)?))
    }

    fn local_addr(&self) -> Result<String> {
        Ok(self.0.local_addr()?.to_string())
    }

    fn accept(&self) -> Result<(StdTcpStream, SocketAddr)> {
        let (stream, addr) = self.0.accept()?;
        Ok((StdTcpStream(stream), addr))
    }
}

pub struct StdTaskSpawner;

impl TaskSpawner for StdTaskSpawner {
    fn spawn_and_forget<T>(task: T)
    where
        T: FnOnce() + Send + 'static,
        T::Output: Send,
    {
        let _ = std::thread::spawn(move || {
            task();
        });
    }
}

pub struct StdServerCodec {
    stream: std::net::TcpStream,
    read_buf: BytesMut,
    write_buf: BytesMut,
    decoder: TspDecoder,
}

impl ServerCodec<StdTcpStream> for StdServerCodec {
    fn from_tcp_stream(stream: StdTcpStream) -> Self {
        Self {
            stream: stream.0,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            decoder: TspDecoder::new(),
        }
    }

    fn send(&mut self, res: Response) -> Result<()> {
        TspEncoder::encode_response(res, &mut self.write_buf)?;
        let bytes_written = self.stream.write(self.write_buf.as_ref())?;
        if bytes_written == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write response",
            )
            .into());
        }
        self.write_buf.advance(bytes_written);
        Ok(self.stream.flush()?)
    }
}

impl Iterator for StdServerCodec {
    type Item = Result<request::Request>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tmp_read_buf = [0_u8; CODEC_READ_BUF_SIZE];

        loop {
            // Try to decode a request from our internal read buffer first
            match self.decoder.decode_request(&mut self.read_buf) {
                Ok(req_opt) => {
                    if let Some(req) = req_opt {
                        return Some(Ok(req));
                    }
                }
                Err(e) => return Some(Err(e)),
            }

            // If we don't have enough data to decode a message, try to read
            // more
            let bytes_read = match self.stream.read(&mut tmp_read_buf) {
                Ok(br) => br,
                Err(e) => return Some(Err(e.into())),
            };
            if bytes_read == 0 {
                // The stream terminated
                return None;
            }
            self.read_buf.extend_from_slice(&tmp_read_buf[..bytes_read]);
        }
    }
}

pub struct StdClientCodec {
    stream: std::net::TcpStream,
    read_buf: BytesMut,
    write_buf: BytesMut,
    decoder: TspDecoder,
}

impl ClientCodec<StdTcpStream> for StdClientCodec {
    fn from_tcp_stream(stream: StdTcpStream) -> Self {
        Self {
            stream: stream.0,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            decoder: TspDecoder::new(),
        }
    }

    fn send(&mut self, req: Request) -> Result<()> {
        TspEncoder::encode_request(req, &mut self.write_buf)?;
        let bytes_written = self.stream.write(self.write_buf.as_ref())?;
        if bytes_written == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write request",
            )
            .into());
        }
        self.write_buf.advance(bytes_written);
        Ok(self.stream.flush()?)
    }
}

impl Iterator for StdClientCodec {
    type Item = Result<response::Response>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tmp_read_buf = [0_u8; CODEC_READ_BUF_SIZE];

        loop {
            // Try to decode a response from our internal read buffer first
            match self.decoder.decode_response(&mut self.read_buf) {
                Ok(res_opt) => {
                    if let Some(res) = res_opt {
                        return Some(Ok(res));
                    }
                }
                Err(e) => return Some(Err(e)),
            }

            // If we don't have enough data to decode a message, try to read
            // more
            let bytes_read = match self.stream.read(&mut tmp_read_buf) {
                Ok(br) => br,
                Err(e) => return Some(Err(e.into())),
            };
            if bytes_read == 0 {
                // The stream terminated
                return None;
            }
            self.read_buf.extend_from_slice(&tmp_read_buf[..bytes_read]);
        }
    }
}

pub struct StdChannelNotify;

impl ChannelNotify for StdChannelNotify {
    type Sender = StdSender<()>;
    type Receiver = StdReceiver<()>;

    fn unbounded() -> (Self::Sender, Self::Receiver) {
        let (tx, rx) = std::sync::mpsc::channel();
        (StdSender(tx), StdReceiver(rx))
    }
}

pub struct StdSender<T>(std::sync::mpsc::Sender<T>);

impl<T> Sender<T> for StdSender<T> {
    fn send(&self, value: T) -> Result<()> {
        self.0
            .send(value)
            .map_err(|e| Error::ChannelSend(e.to_string()))
    }
}

pub struct StdReceiver<T>(std::sync::mpsc::Receiver<T>);

impl<T> Receiver<T> for StdReceiver<T> {
    fn recv(&self) -> Result<T> {
        self.0.recv().map_err(|e| Error::ChannelRecv(e.to_string()))
    }
}
