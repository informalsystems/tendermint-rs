// Copyright (c) 2015 arcnmx

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

#![allow(unsafe_code)]

//! Synchronous in-memory pipe
//!
//! ## Example
//!
//! ```
//! use std::thread::spawn;
//! use std::io::{Read, Write};
//!
//! let (mut read, mut write) = pipe::pipe();
//!
//! let message = "Hello, world!";
//! spawn(move || write.write_all(message.as_bytes()).unwrap());
//!
//! let mut s = String::new();
//! read.read_to_string(&mut s).unwrap();
//!
//! assert_eq!(&s, message);
//! ```

use crossbeam_channel;
use readwrite;

use crossbeam_channel::{Receiver, SendError, Sender, TrySendError};
use std::cmp::min;
use std::hint::unreachable_unchecked;
use std::io::{self, BufRead, Read, Write};
use std::mem::replace;

// value for libstd
const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// The `Read` end of a pipe (see `pipe()`)
pub struct PipeReader {
    receiver: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    position: usize,
}

/// The `Write` end of a pipe (see `pipe()`)
#[derive(Clone)]
pub struct PipeWriter {
    sender: Sender<Vec<u8>>,
}

/// The `Write` end of a pipe (see `pipe()`) that will buffer small writes before sending
/// to the reader end.
pub struct PipeBufWriter {
    sender: Option<Sender<Vec<u8>>>,
    buffer: Vec<u8>,
    size: usize,
}

/// Creates a synchronous memory pipe
pub fn pipe() -> (PipeReader, PipeWriter) {
    let (sender, receiver) = crossbeam_channel::bounded(0);

    (
        PipeReader {
            receiver,
            buffer: Vec::new(),
            position: 0,
        },
        PipeWriter { sender },
    )
}

/// Creates a synchronous memory pipe with buffered writer
pub fn pipe_buffered() -> (PipeReader, PipeBufWriter) {
    let (tx, rx) = crossbeam_channel::bounded(0);

    (
        PipeReader {
            receiver: rx,
            buffer: Vec::new(),
            position: 0,
        },
        PipeBufWriter {
            sender: Some(tx),
            buffer: Vec::with_capacity(DEFAULT_BUF_SIZE),
            size: DEFAULT_BUF_SIZE,
        },
    )
}

/// Creates an asynchronous memory pipe with buffered writer
pub fn async_pipe_buffered() -> (PipeReader, PipeBufWriter) {
    let (tx, rx) = crossbeam_channel::unbounded();

    (
        PipeReader {
            receiver: rx,
            buffer: Vec::new(),
            position: 0,
        },
        PipeBufWriter {
            sender: Some(tx),
            buffer: Vec::with_capacity(DEFAULT_BUF_SIZE),
            size: DEFAULT_BUF_SIZE,
        },
    )
}

/// Creates a pair of pipes for bidirectional communication using buffered writer, a bit like UNIX's `socketpair(2)`.
pub fn async_bipipe_buffered() -> (
    readwrite::ReadWrite<PipeReader, PipeBufWriter>,
    readwrite::ReadWrite<PipeReader, PipeBufWriter>,
) {
    let (r1, w1) = async_pipe_buffered();
    let (r2, w2) = async_pipe_buffered();
    ((r1, w2).into(), (r2, w1).into())
}

fn epipe() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "pipe reader has been dropped")
}

impl PipeWriter {
    /// Extracts the inner `Sender` from the writer
    pub fn into_inner(self) -> Sender<Vec<u8>> {
        self.sender
    }

    /// Gets a reference to the underlying `Sender`
    pub fn sender(&self) -> &Sender<Vec<u8>> {
        &self.sender
    }

    /// Write data to the associated `PipeReader`
    pub fn send<B: Into<Vec<u8>>>(&self, bytes: B) -> io::Result<()> {
        self.sender
            .send(bytes.into())
            .map_err(|_| epipe())
            .map(drop)
    }
}

impl PipeBufWriter {
    #[inline]
    /// Gets a reference to the underlying `Sender`
    pub fn sender(&self) -> &Sender<Vec<u8>> {
        match &self.sender {
            Some(sender) => sender,
            None => unsafe {
                // SAFETY: this is safe as long as `into_inner()` is the only method
                // that clears the sender, and this fn is never called afterward
                unreachable_unchecked()
            },
        }
    }
}

/// Creates a new handle to the `PipeBufWriter` with a fresh new buffer. Any pending data is still
/// owned by the existing writer and should be flushed if necessary.
impl Clone for PipeBufWriter {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            buffer: Vec::with_capacity(self.size),
            size: self.size,
        }
    }
}

/// Creates a new handle to the `PipeReader` with a fresh new buffer. Any pending data is still
/// owned by the existing reader and will not be accessible from the new handle.
impl Clone for PipeReader {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.clone(),
            buffer: Vec::new(),
            position: 0,
        }
    }
}

impl BufRead for PipeReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        while self.position >= self.buffer.len() {
            match self.receiver.recv() {
                // The only existing error is EOF
                Err(_) => break,
                Ok(data) => {
                    self.buffer = data;
                    self.position = 0;
                }
            }
        }

        Ok(&self.buffer[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        debug_assert!(self.buffer.len() - self.position >= amt);
        self.position += amt
    }
}

impl Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let internal = self.fill_buf()?;

        let len = min(buf.len(), internal.len());
        if len > 0 {
            buf[..len].copy_from_slice(&internal[..len]);
            self.consume(len);
        }
        Ok(len)
    }
}

impl Write for &'_ PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let data = buf.to_vec();

        self.send(data).map(|_| buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for PipeWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Write::write(&mut &*self, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut &*self)
    }
}

impl Write for PipeBufWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buffer_len = self.buffer.len();
        let bytes_written = if buf.len() > self.size {
            // bypass buffering for big writes
            buf.len()
        } else {
            // avoid resizing of the buffer
            min(buf.len(), self.size - buffer_len)
        };
        self.buffer.extend_from_slice(&buf[..bytes_written]);

        if self.buffer.len() >= self.size {
            self.flush()?;
        } else {
            // reserve capacity later to avoid needless allocations
            let data = replace(&mut self.buffer, Vec::new());

            // buffer still has space but try to send it in case the other side already awaits
            match self.sender().try_send(data) {
                Ok(_) => self.buffer.reserve(self.size),
                Err(TrySendError::Full(data)) => self.buffer = data,
                Err(TrySendError::Disconnected(data)) => {
                    self.buffer = data;
                    self.buffer.truncate(buffer_len);
                    return Err(epipe());
                }
            }
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            Ok(())
        } else {
            let data = replace(&mut self.buffer, Vec::new());
            match self.sender().send(data) {
                Ok(_) => {
                    self.buffer.reserve(self.size);
                    Ok(())
                }
                Err(SendError(data)) => {
                    self.buffer = data;
                    Err(epipe())
                }
            }
        }
    }
}

/// Flushes the contents of the buffer before the writer is dropped. Errors are ignored, so it is
/// recommended that `flush()` be used explicitly instead of relying on Drop.
///
/// This final flush can be avoided by using `drop(writer.into_inner())`.
impl Drop for PipeBufWriter {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            let data = replace(&mut self.buffer, Vec::new());
            let _ = self.sender().send(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::thread::spawn;

    #[test]
    fn pipe_reader() {
        let i = b"hello there";
        let mut o = Vec::with_capacity(i.len());
        let (mut r, mut w) = pipe();
        let guard = spawn(move || {
            w.write_all(&i[..5]).unwrap();
            w.write_all(&i[5..]).unwrap();
            drop(w);
        });

        r.read_to_end(&mut o).unwrap();
        assert_eq!(i, &o[..]);

        guard.join().unwrap();
    }

    #[test]
    fn pipe_writer_fail() {
        let i = b"hi";
        let (r, mut w) = pipe();
        let guard = spawn(move || {
            drop(r);
        });

        assert!(w.write_all(i).is_err());

        guard.join().unwrap();
    }

    #[test]
    fn small_reads() {
        let block_cnt = 20;
        const BLOCK: usize = 20;
        let (mut r, mut w) = pipe();
        let guard = spawn(move || {
            for _ in 0..block_cnt {
                let data = &[0; BLOCK];
                w.write_all(data).unwrap();
            }
        });

        let mut buff = [0; BLOCK / 2];
        let mut read = 0;
        while let Ok(size) = r.read(&mut buff) {
            // 0 means EOF
            if size == 0 {
                break;
            }
            read += size;
        }
        assert_eq!(block_cnt * BLOCK, read);

        guard.join().unwrap();
    }

    #[test]
    fn pipe_reader_buffered() {
        let i = b"hello there";
        let mut o = Vec::with_capacity(i.len());
        let (mut r, mut w) = pipe_buffered();
        let guard = spawn(move || {
            w.write_all(&i[..5]).unwrap();
            w.write_all(&i[5..]).unwrap();
            w.flush().unwrap();
            drop(w);
        });

        r.read_to_end(&mut o).unwrap();
        assert_eq!(i, &o[..]);

        guard.join().unwrap();
    }

    #[test]
    fn pipe_writer_fail_buffered() {
        let i = &[0; DEFAULT_BUF_SIZE * 2];
        let (r, mut w) = pipe_buffered();
        let guard = spawn(move || {
            drop(r);
        });

        assert!(w.write_all(i).is_err());

        guard.join().unwrap();
    }

    #[test]
    fn small_reads_buffered() {
        let block_cnt = 20;
        const BLOCK: usize = 20;
        let (mut r, mut w) = pipe_buffered();
        let guard = spawn(move || {
            for _ in 0..block_cnt {
                let data = &[0; BLOCK];
                w.write_all(data).unwrap();
            }
            w.flush().unwrap();
        });

        let mut buff = [0; BLOCK / 2];
        let mut read = 0;
        while let Ok(size) = r.read(&mut buff) {
            // 0 means EOF
            if size == 0 {
                break;
            }
            read += size;
        }
        assert_eq!(block_cnt * BLOCK, read);

        guard.join().unwrap();
    }
}
