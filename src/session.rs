//! A session with a validator node

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use types::{PubKeyMsg, TendermintSignable};

use ed25519::{Keyring, PublicKey};
use failure::Error;
use prost::Message;
use rpc::{Request, Response, TendermintResponse};
use secret_connection::SecretConnection;

/// Encrypted session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    connection: SecretConnection<TcpStream>,

    /// Keyring of signature keys
    keyring: Arc<Keyring>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(addr: &str, port: u16, keyring: Arc<Keyring>) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        let connection = SecretConnection::new(socket, keyring.secret_connection_signer())?;
        Ok(Self {
            connection,
            keyring,
        })
    }

    /// Handle an incoming request from the validator
    pub fn handle_request(&mut self) -> Result<bool, Error> {
        println!("handling request ... ");
        let response = match Request::read(&mut self.connection)? {
            Request::SignProposal(req) => self.sign(req)?,
            Request::SignHeartbeat(req) => self.sign(req)?,
            Request::SignVote(req) => self.sign(req)?,
            // non-signable requests:
            Request::ShowPublicKey(ref req) => self.get_public_key(req),
            Request::PoisonPill(_req) => return Ok(false),
        };
        //
        let mut buf = vec![];
        match response {
            Response::SignedHeartBeat(shb) => shb.encode(&mut buf)?,
            Response::SignedProposal(sp) => sp.encode(&mut buf)?,
            Response::SignedVote(sv) => sv.encode(&mut buf)?,
            Response::PublicKey(pk) => pk.encode(&mut buf)?,
        }
        // TODO(ismail): do some proper error handling
        self.connection.write_all(&buf)?;
        Ok(true)
    }

    /// Perform a digital signature operation
    fn sign(&mut self, mut request: impl TendermintResponse) -> Result<Response, Error> {
        // TODO(ismail) figure out if chain_id is a constant / field of the kms or if it should
        // always be embedded with any sign request.

        // TODO(ismail): figure out which key to use here
        // this can't work
        let mut to_sign = vec![];
        request.sign_bytes(&mut to_sign);
        let signer = self.keyring.get_default_signer();

        let sig = self.keyring.sign(&signer.public_key, &to_sign).unwrap();
        request.set_signature(&sig);
        Ok(request.build_response())
    }

    fn get_public_key(&mut self, _request: &PubKeyMsg) -> Response {
        unimplemented!()
    }
}
