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

extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate failure;
extern crate hex;
extern crate hkdf;
extern crate ring;
/// Hacks for accessing the RPC types in tests
extern crate serde_json;
extern crate sha2;
extern crate x25519_dalek;

use prost::Message;
use secret_connection::SecretConnection;
use signatory::{
    encoding::{Decode, Encoding},
    Ed25519PublicKey, Ed25519Seed, Ed25519Signature,
};
use signatory_dalek::Ed25519Signer;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use types::TendermintSignable;

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
}

/// Get the public key associated with the testing private key
fn test_key() -> (Ed25519PublicKey, Ed25519Signer) {
    let seed = Ed25519Seed::decode_from_file("tests/signing.key", Encoding::Raw).unwrap();
    let signer = Ed25519Signer::from(&seed);
    (signatory::public_key(&signer).unwrap(), signer)
}

#[test]
fn test_handle_and_sign_heartbeat() {
    use signatory::ed25519;
    use signatory::Signature;
    use signatory_dalek::Ed25519Verifier;
    use types::heartbeat::{Heartbeat, SignHeartbeatMsg};

    use secret_connection::SecretConnection;
    // this spawns a process which wants to share ephemeral keys and blocks until it reads a reply:
    let mut kms = KmsConnection::create(KMS_TEST_ARGS);

    // we use the same key for both sides:
    let (pub_key, signer) = test_key();
    // Here we reply to the kms with a "remote" ephemeral key, auth signature etc:
    let socket_cp = kms.socket.try_clone().unwrap();
    let public_key = signatory::public_key(&signer).unwrap();
    let mut connection = SecretConnection::new(socket_cp, &public_key, &signer).unwrap();

    // prep a request:
    let addr = vec![
        0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb,
        0x34, 0x46, 0xa8, 0x4b, 0x35,
    ];

    let hb = types::heartbeat::Heartbeat {
        validator_address: addr,
        validator_index: 1,
        height: 15,
        round: 10,
        sequence: 30,
        signature: None,
    };

    let hb_msg = types::heartbeat::SignHeartbeatMsg {
        heartbeat: Some(hb),
    };

    // send request:
    let mut buf = vec![];
    hb_msg.encode(&mut buf).unwrap();
    println!("encoded: {:?}", buf);
    connection.write_all(&buf).unwrap();

    // receive response:
    let mut resp_buf = vec![0u8; 512];
    connection.read(&mut resp_buf).unwrap();

    let actual_len = resp_buf[0];
    let mut resp = vec![0u8; (actual_len + 1) as usize];
    resp.copy_from_slice(&resp_buf[..(actual_len + 1) as usize]);

    let mut hbm: SignHeartbeatMsg =
        SignHeartbeatMsg::decode(&resp).expect("decoding heartbeat failed");
    let mut sign_bytes: Vec<u8> = vec![];
    hbm.sign_bytes(&mut sign_bytes).unwrap();

    let hb: Heartbeat = hbm
        .heartbeat
        .expect("heartbeat should be embedded but none was found");
    let sig: Vec<u8> = hb.signature.expect("expected signature was not found");
    let verifier = Ed25519Verifier::from(&pub_key);
    let signature = Ed25519Signature::from_bytes(sig).unwrap();
    let msg: &[u8] = sign_bytes.as_slice();

    ed25519::verify(&verifier, msg, &signature).unwrap();

    // it all worked; send the kms the message to quit:
    send_poison_pill(&mut kms, &mut connection);
}

fn send_poison_pill(kms: &mut KmsConnection, connection: &mut SecretConnection<TcpStream>) {
    let pill = types::PoisonPillMsg {};
    let mut buf = vec![];
    pill.encode(&mut buf).unwrap();
    connection.write_all(&buf).unwrap();
    println!("sent poison pill");
    kms.process.wait().unwrap();
}

#[test]
fn test_handle_poisonpill() {
    use secret_connection::SecretConnection;
    // this spawns a process which wants to share ephemeral keys and blocks until it reads a reply:
    let mut kms = KmsConnection::create(KMS_TEST_ARGS);

    // we use the same key for both sides:
    let (pub_key, signer) = test_key();
    // Here we reply to the kms with a "remote" ephemeral key, auth signature etc:
    let socket_cp = kms.socket.try_clone().unwrap();
    let mut connection = SecretConnection::new(socket_cp, &pub_key, &signer).unwrap();

    // use the secret connection to send a message
    send_poison_pill(&mut kms, &mut connection);
}
