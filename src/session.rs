//! A session with a validator node

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use types::TendermintSign;

use ed25519::{Keyring, PublicKey};
use failure::Error;
use rpc::{Request, Response};
use secret_connection::SecretConnection;
use signatory::providers::dalek::Ed25519Signer as DalekSigner;

/// A (soon-to-be-encrypted) session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    connection: SecretConnection<TcpStream>,

    /// Keyring of signature keys
    keyring: Arc<Keyring>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(
        addr: &str,
        port: u16,
        keyring: Arc<Keyring>,
        secret_connection_key: Arc<DalekSigner>,
    ) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        let connection = SecretConnection::new(socket, &secret_connection_key)?;
        Ok(Self {
            connection,
            keyring,
        })
    }

    /// Handle an incoming request from the validator
    pub fn handle_request(&mut self) -> Result<bool, Error> {
        let response = match Request::read(&mut self.connection)? {
            Request::SignProposal(req) => self.sign(req)?,
            _ => return Ok(false),
            #[cfg(debug_assertions)]
            Request::PoisonPill => return Ok(false),
        };

        // self.connection.write_all(&response.to_vec())?;
        Ok(true)
    }

    /// Perform a digital signature operation
    fn sign(&mut self, request: impl TendermintSign) -> Result<(), Error> {
        unimplemented!()
    }
}
