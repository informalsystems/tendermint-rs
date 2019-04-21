//! A session with a validator node

use crate::{
    chain,
    error::{
        KmsError,
        KmsErrorKind::{ExceedMaxHeight, VerificationError},
    },
    prost::Message,
    rpc::{Request, Response, TendermintRequest},
    unix_connection::UnixConnection,
};
use signatory::{ed25519, PublicKeyed};
use signatory_dalek::Ed25519Signer;
use std::{
    fmt::Debug,
    io::{Read, Write},
    marker::{Send, Sync},
    net::TcpStream,
    os::unix::net::UnixStream,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use subtle::ConstantTimeEq;
use tendermint::{
    amino_types::{PingRequest, PingResponse, PubKeyRequest, PubKeyResponse},
    node,
    secret_connection::{self, SecretConnection},
};

/// Encrypted session with a validator node
pub struct Session<Connection> {
    /// Chain ID for this session
    chain_id: chain::Id,

    // Do not sign blocks greates than this height
    max_height: Option<tendermint::block::Height>,

    /// TCP connection to a validator node
    connection: Connection,
}

impl Session<SecretConnection<TcpStream>> {
    /// Create a new session with the validator at the given address/port
    pub fn connect_tcp(
        chain_id: chain::Id,
        max_height: Option<tendermint::block::Height>,
        validator_peer_id: Option<node::Id>,
        host: &str,
        port: u16,
        secret_connection_key: &ed25519::Seed,
    ) -> Result<Self, KmsError> {
        debug!("{}: Connecting to {}:{}...", chain_id, host, port);

        let socket = TcpStream::connect(format!("{}:{}", host, port))?;
        let signer = Ed25519Signer::from(secret_connection_key);
        let public_key = secret_connection::PublicKey::from(signer.public_key()?);
        let connection = SecretConnection::new(socket, &public_key, &signer)?;
        let actual_peer_id = connection.remote_pubkey().peer_id();

        // TODO(tarcieri): move this logic into `SecretConnection::new`?
        if let Some(expected_peer_id) = validator_peer_id {
            if expected_peer_id.ct_eq(&actual_peer_id).unwrap_u8() == 0 {
                fail!(
                    VerificationError,
                    "{}:{}: validator peer ID mismatch! (expected {}, got {})",
                    host,
                    port,
                    expected_peer_id,
                    actual_peer_id
                );
            }
        } else {
            // TODO(tarcieri): make peer verification mandatory
            warn!(
                "[{}] {}:{}: unverified validator peer ID! ({})",
                chain_id, host, port, actual_peer_id
            );
        }

        Ok(Self {
            chain_id,
            max_height,
            connection,
        })
    }
}

impl Session<UnixConnection<UnixStream>> {
    pub fn connect_unix(
        chain_id: chain::Id,
        max_height: Option<tendermint::block::Height>,
        socket_path: &Path,
    ) -> Result<Self, KmsError> {
        debug!(
            "{}: Connecting to socket at {}...",
            chain_id,
            socket_path.to_str().unwrap()
        );

        let socket = UnixStream::connect(socket_path)?;
        let connection = UnixConnection::new(socket);

        Ok(Self {
            chain_id,
            max_height,
            connection,
        })
    }
}

impl<Connection> Session<Connection>
where
    Connection: Read + Write + Sync + Send,
{
    /// Main request loop
    pub fn request_loop(&mut self, should_term: &Arc<AtomicBool>) -> Result<(), KmsError> {
        debug!("starting handle request loop ... ");
        while self.handle_request(should_term)? {}
        Ok(())
    }

    /// Handle an incoming request from the validator
    fn handle_request(&mut self, should_term: &Arc<AtomicBool>) -> Result<bool, KmsError> {
        if should_term.load(Ordering::Relaxed) {
            info!("terminate signal received");
            return Ok(false);
        }
        debug!("started handling request ... ");
        let response = match Request::read(&mut self.connection)? {
            Request::SignProposal(req) => self.sign(req)?,
            Request::SignVote(req) => self.sign(req)?,
            // non-signable requests:
            Request::ReplyPing(ref req) => self.reply_ping(req),
            Request::ShowPublicKey(ref req) => self.get_public_key(req)?,
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

        let registry = chain::REGISTRY.get();
        let chain = registry.get_chain(&self.chain_id).unwrap();

        if let Some(request_state) = request.consensus_state() {
            // TODO(tarcieri): better handle `PoisonError`?
            let mut chain_state = chain.state.lock().unwrap();
            chain_state.update_consensus_state(request_state.clone())?;
        }

        if let Some(max_height) = self.max_height {
            if let Some(height) = request.height() {
                if height > max_height.value() as i64 {
                    fail!(
                        ExceedMaxHeight,
                        "attempted to sign at height {} which is greater than {}",
                        height,
                        max_height,
                    );
                }
            }
        }

        let mut to_sign = vec![];
        request.sign_bytes(self.chain_id, &mut to_sign)?;

        // TODO(ismail): figure out which key to use here instead of taking the only key
        // from keyring here:
        let sig = chain.keyring.sign_ed25519(None, &to_sign)?;

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
        let registry = chain::REGISTRY.get();
        let chain = registry.get_chain(&self.chain_id).unwrap();

        Ok(Response::PublicKey(PubKeyResponse::from(
            *chain.keyring.default_pubkey()?,
        )))
    }
}
