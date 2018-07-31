//! KMS integration test

#[macro_use]
extern crate serde_derive;
extern crate signatory;

use signatory::ed25519::{self, FromSeed, Signer};
use signatory::providers::dalek;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};

/// Address the mock validator listens on
pub const MOCK_VALIDATOR_ADDR: &str = "127.0.0.1";

/// Port the mock validator listens on
pub const MOCK_VALIDATOR_PORT: u16 = 23456;

/// Arguments to pass when launching the KMS
pub const KMS_TEST_ARGS: &[&str] = &["run", "-c", "tests/test.toml"];

/// Hacks for accessing the RPC types in tests
mod rpc {
    include!("../src/rpc.rs");
}

use rpc::*;

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
    pub fn sign(&mut self, public_key: &ed25519::PublicKey, msg: &[u8]) -> ed25519::Signature {
        let req = Request::Sign(SignRequest {
            public_key: public_key.as_bytes().to_vec(),
            msg: msg.to_owned(),
        });

        self.socket.write_all(&req.to_vec()).unwrap();

        match Response::read(&mut self.socket).unwrap() {
            Response::Sign(ref response) => ed25519::Signature::from_bytes(&response.sig).unwrap(),
        }
    }

    /// Instruct the KMS to terminate by sending it the "poison pill" message
    pub fn terminate(mut self) {
        self.socket
            .write_all(&Request::PoisonPill.to_vec())
            .unwrap();
        self.process.wait().unwrap();
    }
}

/// Get the public key associated with the testing private key
fn test_public_key() -> ed25519::PublicKey {
    let mut file = File::open("tests/test.key").unwrap();
    let mut key_material = vec![];
    file.read_to_end(key_material.as_mut()).unwrap();

    let seed = ed25519::Seed::from_slice(&key_material).unwrap();
    let signer = dalek::Ed25519Signer::from_seed(seed);
    signer.public_key().unwrap()
}

#[test]
fn test_sign() {
    let mut kms = KmsConnection::create(KMS_TEST_ARGS);

    let test_message = b"Hello, world!";
    let pubkey = test_public_key();
    let signature = kms.sign(&pubkey, test_message);

    // Ensure the signature verifies
    pubkey.verify(test_message, &signature).unwrap();

    kms.terminate();
}
