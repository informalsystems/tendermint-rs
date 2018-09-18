//! KMS integration test

// TODO: get rid of hacks for using RPC types in tests
#![allow(unused_imports, unused_variables, dead_code)]

#[macro_use]
extern crate abscissa_derive;
#[macro_use]
extern crate failure_derive;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate rand;
extern crate signatory;
extern crate signatory_dalek;

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

use prost::Message;
use signatory::{
    encoding::{Decode, Encoding},
    Ed25519PublicKey, Ed25519Seed, Ed25519Signature, Signer,
};
use signatory_dalek::Ed25519Signer;
use std::ffi::OsStr;
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
        public_key: &Ed25519PublicKey,
        signer: &Signer<Ed25519Signature>,
        request: impl types::TendermintSign,
    ) -> Ed25519Signature {
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
        signatory::sign(signer, &json_msg.into_bytes()).unwrap()
    }
}

/// Get the public key associated with the testing private key
fn test_key() -> (Ed25519PublicKey, Ed25519Signer) {
    let seed = Ed25519Seed::decode_from_file("tests/signing.key", Encoding::Raw).unwrap();
    let signer = Ed25519Signer::from(&seed);
    (signatory::public_key(&signer).unwrap(), signer)
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
    let public_key = signatory::public_key(&signer).unwrap();
    let mut connection = SecretConnection::new(socket_cp, &public_key, &signer).unwrap();

    // use the secret connection to send a message
    let pill = types::PoisonPillMsg {};
    let mut buf = vec![];
    pill.encode(&mut buf).unwrap();
    connection.write_all(&buf).unwrap();
    println!("sent poison pill");
    kms.process.wait().unwrap();
}
