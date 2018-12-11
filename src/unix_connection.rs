use std::io;
use std::marker::{Send, Sync};

use crate::error::KmsError;

/// Protocol implementation of the UNIX socket domain connection
pub struct UnixConnection<IoHandler> {
    socket: IoHandler,
}

impl<IoHandler> UnixConnection<IoHandler>
where
    IoHandler: io::Read + io::Write + Send + Sync,
{
    #[allow(clippy::new_ret_no_self)]
    /// Create a new `UnixConnection` for the given socket
    pub fn new(socket: IoHandler) -> Result<Self, KmsError> {
        Ok(Self { socket })
    }
}

impl<IoHandler> io::Read for UnixConnection<IoHandler>
where
    IoHandler: io::Read + io::Write + Send + Sync,
{
    fn read(&mut self, data: &mut [u8]) -> Result<usize, io::Error> {
        self.socket.read(data)
    }
}

impl<IoHandler> io::Write for UnixConnection<IoHandler>
where
    IoHandler: io::Read + io::Write + Send + Sync,
{
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.socket.write(data)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.socket.flush()
    }
}
