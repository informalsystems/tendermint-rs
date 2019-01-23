//! A session with a validator node

use signatory::ed25519;
use signatory_dalek::Ed25519Signer;
use std::{
    fmt::Debug,
    io::{Read, Write},
    marker::{Send, Sync},
    net::TcpStream,
    os::unix::net::UnixStream,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};
use tendermint::{
    amino_types::{PingRequest, PingResponse, PubKeyRequest, PubKeyResponse},
    chain,
    public_keys::SecretConnectionKey,
    SecretConnection,
};

use crate::{
    error::KmsError,
    keyring::KeyRing,
    prost::Message,
    rpc::{Request, Response, TendermintRequest},
    unix_connection::UnixConnection,
};

/// Encrypted session with a validator node
pub struct Session<Connection> {
    /// Chain ID for this session
    chain_id: chain::Id,

    /// TCP connection to a validator node
    connection: Connection,
}

impl Session<SecretConnection<TcpStream>> {
    /// Create a new session with the validator at the given address/port
    pub fn connect_tcp(
        chain_id: chain::Id,
        host: &str,
        port: u16,
        secret_connection_key: &ed25519::Seed,
    ) -> Result<Self, KmsError> {
        debug!("{}: Connecting to {}:{}...", chain_id, host, port);

        let socket = TcpStream::connect(format!("{}:{}", host, port))?;
        let signer = Ed25519Signer::from(secret_connection_key);
        let public_key = SecretConnectionKey::from(ed25519::public_key(&signer)?);
        let connection = SecretConnection::new(socket, &public_key, &signer)?;

        Ok(Self {
            chain_id,
            connection,
        })
    }
}

impl Session<UnixConnection<UnixStream>> {
    pub fn connect_unix(chain_id: chain::Id, socket_path: &Path) -> Result<Self, KmsError> {
        debug!(
            "{}: Connecting to socket at {}...",
            chain_id,
            socket_path.to_str().unwrap()
        );

        let socket = UnixStream::connect(socket_path)?;
        let connection = UnixConnection::new(socket);

        Ok(Self {
            chain_id,
            connection,
        })
    }
}

impl<Connection> Session<Connection>
where
    Connection: Read + Write + Sync + Send,
{
    /// Main request loop
    pub fn request_loop(&mut self, should_term: Arc<AtomicBool>) -> Result<(), KmsError> {
        debug!("starting handle request loop ... ");
        while self.handle_request()? {}
        // Only happens when we received a PoisonPillMsg, so tell the outer
        // thread to terminate.
        should_term.swap(true, Ordering::Relaxed);
        Ok(())
    }

    /// Handle an incoming request from the validator
    fn handle_request(&mut self) -> Result<bool, KmsError> {
        debug!("started handling request ... ");
        let response = match Request::read(&mut self.connection)? {
            Request::SignProposal(req) => self.sign(req)?,
            Request::SignVote(req) => self.sign(req)?,
            // non-signable requests:
            Request::ReplyPing(ref req) => self.reply_ping(req),
            Request::ShowPublicKey(ref req) => self.get_public_key(req)?,
            Request::PoisonPill(_req) => return Ok(false),
        };

        let mut buf = vec![];

        match response {
            Response::SignedProposal(sp) => sp.encode(&mut buf)?,
            Response::SignedVote(sv) => sv.encode(&mut buf)?,
            Response::Ping(ping) => ping.encode(&mut buf)?,
            Response::PublicKey(pk) => pk.encode(&mut buf)?,
        }

        self.connection.write_all(&buf)?;
        debug!("... success handling request");
        Ok(true)
    }

    /// Perform a digital signature operation
    fn sign<T: TendermintRequest + Debug>(&mut self, mut request: T) -> Result<Response, KmsError> {
        request.validate()?;

        let mut to_sign = vec![];
        request.sign_bytes(self.chain_id, &mut to_sign)?;

        // TODO(ismail): figure out which key to use here instead of taking the only key
        // from keyring here:
        let sig = KeyRing::sign(None, &to_sign)?;

        request.set_signature(&sig);
        debug!("successfully signed request:\n {:?}", request);
        Ok(request.build_response())
    }

    /// Reply to a ping request
    fn reply_ping(&mut self, _request: &PingRequest) -> Response {
        debug!("replying with PingResponse");
        Response::Ping(PingResponse {})
    }

    /// Get the public key for (the only) public key in the keyring
    fn get_public_key(&mut self, _request: &PubKeyRequest) -> Result<Response, KmsError> {
        let pubkey = KeyRing::default_pubkey()?;
        let pubkey_bytes = pubkey.as_bytes();

        Ok(Response::PublicKey(PubKeyResponse {
            pub_key_ed25519: pubkey_bytes.to_vec(),
        }))
    }
}
