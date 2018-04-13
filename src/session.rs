//! A session with a validator node

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use error::Error;
use ed25519::Keyring;

/// A (soon-to-be-encrypted) session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    socket: TcpStream,

    /// Keyring of signature keys
    keyring: Arc<Keyring>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(addr: &str, port: u16, keyring: Arc<Keyring>) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);

        let mut socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        socket.write_all(b"HELLO\n")?;

        Ok(Self { socket, keyring })
    }

    /// Handle incoming requests from the validator
    pub fn handle_requests(&self) -> Result<(), Error> {
        // TODO: actually do stuff
        Ok(())
    }
}
