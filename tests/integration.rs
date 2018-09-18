//! KMS integration test

#[allow(unused_imports)]
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
#[allow(unused_imports)]
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

    /// Sign the given message with the given public key using the KMS
    pub fn sign(
        &mut self,
        public_key: &Ed25519PublicKey,
        signer: &Signer<Ed25519Signature>,
        mut request: impl types::TendermintSignable,
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
        let mut to_sign = vec![];
        request.sign_bytes(&mut to_sign);
        signatory::sign(signer, &mut to_sign).unwrap()
    }
}

/// Get the public key associated with the testing private key
fn test_key() -> (Ed25519PublicKey, Ed25519Signer) {
    let seed = Ed25519Seed::decode_from_file("tests/signing.key", Encoding::Raw).unwrap();
    let signer = Ed25519Signer::from(&seed);
    (signatory::public_key(&signer).unwrap(), signer)
}

#[test]
fn test_handle_sign_request() {
    use signatory_dalek::{Ed25519Signer, Ed25519Verifier};
    use signatory::ed25519;
    use signatory::Signature;

    use secret_connection::SecretConnection;
    // this spawns a process which wants to share ephemeral keys and blocks until it reads a reply:
    let mut kms = KmsConnection::create(KMS_TEST_ARGS);

    // we use the same key for both sides:
    let (pub_key, signer) = test_key();
    // Here we reply to the kms with a "remote" ephemeral key, auth signature etc:
    let socket_cp = kms.socket.try_clone().unwrap();
    let mut connection = SecretConnection::new(socket_cp, &signer).unwrap();

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

    // recv response:
    let mut resp_buf = vec![0u8; 512];
    connection.read(&mut resp_buf).unwrap();

    let actual_len = resp_buf[0];
    let mut resp = vec![0u8; (actual_len+1) as usize];
    resp.copy_from_slice(&resp_buf[..(actual_len+1) as usize]);

    match types::heartbeat::SignHeartbeatMsg::decode(&resp) {
        Ok(mut hbm) => {
            println!("yay");
            // verify signature:
            let mut sign_bytes = vec![];
            hbm.sign_bytes(&mut sign_bytes);
            println!("this was signed: {:?}", sign_bytes);
            if let Some(ref mut hb) = hbm.heartbeat {
                if let Some(ref sig) =  hb.signature {
                    let verifier = Ed25519Verifier::from(&pub_key);
                    let signature = Ed25519Signature::from_bytes(sig).unwrap();
                    assert!(ed25519::verify(&verifier, &sign_bytes, &signature).is_ok());
                } else { panic!("no signature included"); }
            } else { panic!("no heartbeat embedded"); }


        },
        Err(e) => println!("nay {}", e),
    }


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
    let (_, signer) = test_key();
    // Here we reply to the kms with a "remote" ephemeral key, auth signature etc:
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
