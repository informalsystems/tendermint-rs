//! A session with a validator node

use std::io::Write;
use std::net::TcpStream;

use error::Error;

/// A (soon-to-be-encrypted) session with a validator node
pub struct Session {
    socket: TcpStream,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(addr: &str, port: u16) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let mut session = Self {
            socket: TcpStream::connect(format!("{}:{}", addr, port))?,
        };

        // TODO: advertise keyring
        session.hello()?;
        Ok(session)
    }

    /// Handle incoming requests from the validator
    pub fn handle_requests(&self) -> Result<(), Error> {
        // TODO: actually do stuff
        Ok(())
    }

    fn hello(&mut self) -> Result<(), Error> {
        self.socket.write_all(b"HELLO\n")?;
        Ok(())
    }
}
