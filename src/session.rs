//! A session with a validator node

use signatory::{ed25519, Ed25519Seed};
use signatory_dalek::Ed25519Signer;
use std::io::Write;
use std::net::TcpStream;
use types::{PingRequest, PingResponse, PubKeyMsg};

use ed25519::keyring::KeyRing;
use error::KmsError;
use prost::Message;
use rpc::{Request, Response, TendermintResponse};
use tm_secret_connection::SecretConnection;

/// Encrypted session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    connection: SecretConnection<TcpStream>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(
        addr: &str,
        port: u16,
        secret_connection_key: &Ed25519Seed,
    ) -> Result<Self, KmsError> {
        debug!("Connecting to {}:{}...", addr, port);
        let socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        let signer = Ed25519Signer::from(secret_connection_key);
        let public_key = ed25519::public_key(&signer)?;
        let connection = SecretConnection::new(socket, &public_key, &signer)?;
        Ok(Self { connection })
    }

    /// Handle an incoming request from the validator
    pub fn handle_request(&mut self) -> Result<bool, KmsError> {
        let response = match Request::read(&mut self.connection)? {
            Request::SignProposal(req) => self.sign(req)?,
            Request::SignHeartbeat(req) => self.sign(req)?,
            Request::SignVote(req) => self.sign(req)?,
            // non-signable requests:
            Request::ReplyPing(ref req) => self.reply_ping(req),
            Request::ShowPublicKey(ref req) => self.get_public_key(req)?,
            Request::PoisonPill(_req) => return Ok(false),
        };
        //
        let mut buf = vec![];
        match response {
            Response::SignedHeartBeat(shb) => shb.encode(&mut buf)?,
            Response::SignedProposal(sp) => sp.encode(&mut buf)?,
            Response::SignedVote(sv) => sv.encode(&mut buf)?,
            Response::Ping(ping) => ping.encode(&mut buf)?,
            Response::PublicKey(pk) => pk.encode(&mut buf)?,
        }
        self.connection.write_all(&buf)?;
        Ok(true)
    }

    /// Perform a digital signature operation
    fn sign(&mut self, mut request: impl TendermintResponse) -> Result<Response, KmsError> {
        let mut to_sign = vec![];
        // TODO(ismail): this should either be a config param, or, included in the request!
        let chain_id = "test_chain_id";
        request.sign_bytes(chain_id, &mut to_sign)?;
        // TODO(ismail): figure out which key to use here instead of taking the only key
        // from keyring here:
        let sig = KeyRing::sign(None, &to_sign)?;

        request.set_signature(&sig);
        Ok(request.build_response())
    }
    fn reply_ping(&mut self, _request: &PingRequest) -> Response {
        Response::Ping(PingResponse {})
    }

    fn get_public_key(&mut self, _request: &PubKeyMsg) -> Result<Response, KmsError> {
        let pubkey = KeyRing::get_only_signing_pubkey()?;
        let pubkey_bytes = pubkey.as_bytes();

        Ok(Response::PublicKey(PubKeyMsg {
            pub_key_ed25519: pubkey_bytes.to_vec(),
        }))
    }
}
