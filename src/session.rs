//! A session with a validator node

use signatory::{ed25519, Ed25519Seed};
use signatory_dalek::Ed25519Signer;
use std::io::Write;
use std::net::TcpStream;
use types::{PubKeyMsg, TendermintSign};

use error::Error;
use prost::Message;
use rpc::{Request, Response};
use secret_connection::SecretConnection;

/// Encrypted session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    connection: SecretConnection<TcpStream>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(addr: &str, port: u16, secret_connection_key: &Ed25519Seed) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        let signer = Ed25519Signer::from(secret_connection_key);
        let public_key = ed25519::public_key(&signer)?;
        let connection = SecretConnection::new(socket, &public_key, &signer)?;
        Ok(Self { connection })
    }

    /// Handle an incoming request from the validator
    pub fn handle_request(&mut self) -> Result<bool, Error> {
        println!("handling request ... ");
        let response = match Request::read(&mut self.connection)? {
            Request::SignProposal(req) => self.sign(req)?,
            Request::SignHeartbeat(req) => self.sign(req)?,
            Request::SignVote(req) => self.sign(req)?,
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
    fn sign(&mut self, request: impl TendermintSign) -> Result<Response, Error> {
        // TODO(ismail) figure out if chain_id is a constant / field of the kms?
        let chain_id = "TODO";
        let _json = request.cannonicalize(chain_id);
        // TODO(ismail): figure out which key to use here
        //match self.keyring.sign( &PublicKey(vec![]), &json.into_bytes()) { }
        unimplemented!()
    }

    fn get_public_key(&mut self, _request: &PubKeyMsg) -> Response {
        unimplemented!()
    }
}
