//! UNIX IPC wire protocol implementation

use std::io;
use std::fs;
use std::path::PathBuf;
use std::os::unix::net::{UnixStream, UnixListener};

use error::Error;

pub struct UNIXConnection {
    path: PathBuf,
    listener: UnixListener,
    socket: UnixStream,
}

impl UNIXConnection {
    pub fn new(socket_path: &PathBuf) -> Result<Self, Error> {
        // Try to unlink the socket path, shouldn't fail if it doesn't exist
        if let Err(e) = fs::remove_file(socket_path) {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(Error::from(e));
            }
        }

        // Create a listener and wait for a connection
        let path = socket_path.clone();
        let listener = UnixListener::bind(&path)?;
        let (socket, _addr) = listener.accept()?;

        Ok(Self { path, listener, socket })
    }
}

impl io::Read for UNIXConnection {

    fn read (&mut self, data: &mut [u8]) -> Result<usize, io::Error> {
        self.socket.read(data)
    }
}

impl io::Write for UNIXConnection {

    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.socket.write(data)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.socket.flush()
    }
}
