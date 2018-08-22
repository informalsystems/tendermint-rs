//! KMS integration test

extern crate prost;
#[macro_use]
extern crate prost_derive;

extern crate rand;
extern crate signatory;

use prost::Message;
use signatory::ed25519::{self, FromSeed, Signer};
use signatory::providers::dalek;
use std::ffi::OsStr;
use std::fs::File;
#[allow(unused_imports)]
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use types::TendermintSign;

/// Address the mock validator listens on
pub const MOCK_VALIDATOR_ADDR: &str = "127.0.0.1";

/// Port the mock validator listens on
pub const MOCK_VALIDATOR_PORT: u16 = 23456;

/// Arguments to pass when launching the KMS
pub const KMS_TEST_ARGS: &[&str] = &["run", "-c", "tests/test.toml"];

/// Hacks for accessing the RPC types in tests
#[macro_use]
extern crate serde_json;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate failure;
extern crate hex;
extern crate hkdf;
extern crate ring;
extern crate sha2;
extern crate x25519_dalek;
#[macro_use]
extern crate failure_derive;

mod types {
    include!("../src/types/mod.rs");
}

#[macro_use]
mod error {
    include!("../src/error.rs");
}

mod secret_connection {
    include!("../src/secret_connection.rs");
}

/// Receives incoming KMS connection then sends commands
struct KmsConnection {
    /// KMS child process
    process: Child,

    /// TCP socket to KMS process
    socket: TcpStream,
}

impl KmsConnection {
    /// Spawn the KMS process and wait for an incoming connection
    pub fn create<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let listener =
            TcpListener::bind(format!("{}:{}", MOCK_VALIDATOR_ADDR, MOCK_VALIDATOR_PORT)).unwrap();

        let process = Command::new("./target/debug/cosmos-kms")
            .args(args)
            .spawn()
            .unwrap();

        let (socket, _) = listener.accept().unwrap();
        Self { process, socket }
    }

    /// Sign the given message with the given public key using the KMS
    pub fn sign(
        &mut self,
        public_key: &ed25519::PublicKey,
        signer: signatory::providers::dalek::Ed25519Signer,
        request: impl types::TendermintSign,
    ) -> ed25519::Signature {
        // TODO(ismail) SignRequest ->  now one of:
        // SignHeartbeat(SignHeartbeatMsg), SignProposal(SignProposalMsg), SignVote(SignVoteMsg), ShowPublicKey(PubKeyMsg),
        /*let req = Request::SignHeartbeat(types::heartbeat::SignHeartbeatMsg {
            public_key: public_key.as_bytes().to_vec(),
            msg: msg.to_owned(),
        });

        self.socket.write_all(&req.to_vec()).unwrap();

        match Response::read(&mut self.socket).unwrap() {
            Response::Sign(ref response) => ed25519::Signature::from_bytes(&response.sig).unwrap(),
        }*/
        let json_msg = request.cannonicalize("chain_id");
        signer.sign(&json_msg.into_bytes()).unwrap()
    }
}

/// Get the public key associated with the testing private key
fn test_key() -> (
    ed25519::PublicKey,
    signatory::providers::dalek::Ed25519Signer,
) {
    let mut file = File::open("tests/test.key").unwrap();
    let mut key_material = vec![];
    file.read_to_end(key_material.as_mut()).unwrap();

    let seed = ed25519::Seed::from_slice(&key_material).unwrap();
    let signer = dalek::Ed25519Signer::from_seed(seed);
    (signer.public_key().unwrap(), signer)
}

#[test]
fn test_handle_poisonpill() {
    use secret_connection::SecretConnection;
    // this spawns a process which wants to share ephermal keys and blocks until it reads a reply:
    let mut kms = KmsConnection::create(KMS_TEST_ARGS);

    // we use the same key for both sides:
    let (_, signer) = test_key();
    // Here we reply to the kms with a "remote" ephermal key, auth signature etc:
    let socket_cp = kms.socket.try_clone().unwrap();
    let mut connection = SecretConnection::new(socket_cp, &signer).unwrap();

    // use the secret connection to send a message
    let pill = types::PoisonPillMsg {};
    let mut buf = vec![];
    pill.encode(&mut buf).unwrap();
    connection.write_all(&buf).unwrap();
    println!("sent poison pill");
    kms.process.wait().unwrap();
}
