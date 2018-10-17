use std::io;
use std::marker::{Send, Sync};

use error::Error;

pub struct UNIXConnection<IoHandler> {
    socket: IoHandler,
}

impl<IoHandler: io::Read + io::Write + Send + Sync>  UNIXConnection<IoHandler> {
    pub fn new(socket: IoHandler) -> Result<Self, Error> {
        Ok(Self { socket })
    }
}

impl<IoHandler> io::Read for UNIXConnection<IoHandler>
where
    IoHandler: io::Read + io::Write + Send + Sync,
{

    fn read (&mut self, data: &mut [u8]) -> Result<usize, io::Error> {
        self.socket.read(data)
    }
}

impl<IoHandler> io::Write for UNIXConnection<IoHandler>
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
