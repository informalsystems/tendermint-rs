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

//! Asynchronous in-memory pipe

use std::cmp::min;
use std::io::{self, BufRead, Read, Write};

use flume::{self, Receiver, SendError, Sender, TrySendError};

// value for libstd
const DEFAULT_BUF_SIZE: usize = 8 * 1024;

/// The `Read` end of a pipe (see `pipe()`)
pub struct Reader {
    receiver: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    position: usize,
}

/// The `Write` end of a pipe (see `pipe()`) that will buffer small writes before sending
/// to the reader end.
pub struct BufWriter {
    sender: Option<Sender<Vec<u8>>>,
    buffer: Vec<u8>,
    size: usize,
}

/// Creates an asynchronous memory pipe with buffered writer
pub fn async_pipe_buffered() -> (Reader, BufWriter) {
    let (tx, rx) = flume::unbounded();

    (
        Reader {
            receiver: rx,
            buffer: Vec::new(),
            position: 0,
        },
        BufWriter {
            sender: Some(tx),
            buffer: Vec::with_capacity(DEFAULT_BUF_SIZE),
            size: DEFAULT_BUF_SIZE,
        },
    )
}

/// Creates a pair of pipes for bidirectional communication using buffered writer, a bit like UNIX's
/// `socketpair(2)`.
pub fn async_bipipe_buffered() -> (
    readwrite::ReadWrite<Reader, BufWriter>,
    readwrite::ReadWrite<Reader, BufWriter>,
) {
    let (r1, w1) = async_pipe_buffered();
    let (r2, w2) = async_pipe_buffered();
    ((r1, w2).into(), (r2, w1).into())
}

fn epipe() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "pipe reader has been dropped")
}

impl BufWriter {
    #[inline]
    /// Gets a reference to the underlying `Sender`
    pub fn sender(&self) -> &Sender<Vec<u8>> {
        // SAFETY: this is safe as long as `into_inner()` is the only method
        // that clears the sender, and this fn is never called afterward
        self.sender.as_ref().expect("sender to be present")
    }
}

impl BufRead for Reader {
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

impl Read for Reader {
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

impl Write for BufWriter {
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
            let data = std::mem::take(&mut self.buffer);

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
            let data = std::mem::take(&mut self.buffer);
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
impl Drop for BufWriter {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            let data = std::mem::take(&mut self.buffer);
            self.sender().send(data).ok();
        }
    }
}
