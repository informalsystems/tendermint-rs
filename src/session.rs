//! A session with a validator node

use signatory::{ed25519, Ed25519Seed};
use signatory_dalek::Ed25519Signer;
use std::marker::{Send, Sync};
use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::{fs, io};
use types::{PubKeyMsg, TendermintSign};

use error::Error;
use prost::Message;
use rpc::{Request, Response};
use secret_connection::SecretConnection;
use unix_connection::UNIXConnection;

/// Encrypted session with a validator node
pub struct Session<Connection> {
    /// TCP connection to a validator node
    connection: Connection,
}

impl Session<SecretConnection<TcpStream>> {
    /// Create a new session with the validator at the given address/port
    pub fn new_seccon(
        addr: &str,
        port: u16,
        secret_connection_key: &Ed25519Seed,
    ) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        let signer = Ed25519Signer::from(secret_connection_key);
        let public_key = ed25519::public_key(&signer)?;
        let connection = SecretConnection::new(socket, &public_key, &signer)?;
        Ok(Self { connection })
    }
}

impl Session<UNIXConnection<UnixStream>> {
    pub fn new_unix(socket_path: &PathBuf) -> Result<Self, Error> {
        // Try to unlink the socket path, shouldn't fail if it doesn't exist
        if let Err(e) = fs::remove_file(socket_path) {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(Error::from(e));
            }
        }

        debug!(
            "Waiting for a connection at {}...",
            socket_path.to_str().unwrap()
        );

        let listener = UnixListener::bind(&socket_path)?;
        let (socket, addr) = listener.accept()?;

        debug!("Accepted connection from {:?}", addr);
        debug!("Stopped listening on {}", socket_path.to_str().unwrap());

        let connection = UNIXConnection::new(socket)?;
        Ok(Self { connection })
    }
}

impl<Connection: io::Read + io::Write + Sync + Send> Session<Connection> {
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
