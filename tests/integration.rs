//! KMS integration test

// TODO: get rid of hacks for using RPC types in tests
#![allow(unused_imports, unused_variables, dead_code)]

#[macro_use]
extern crate abscissa;
#[macro_use]
extern crate abscissa_derive;
#[macro_use]
extern crate failure_derive;
extern crate prost_amino as prost;
#[macro_use]
extern crate prost_amino_derive as prost_derive;
extern crate rand;
extern crate signatory;

extern crate signatory_dalek;
#[cfg(feature = "yubihsm")]
extern crate signatory_yubihsm;
extern crate subtle_encoding;
extern crate tempfile;

extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate failure;
extern crate sha2;
extern crate tm_secret_connection;

use chrono::{DateTime, Utc};
use prost::Message;
use rand::Rng;
use signatory::{ed25519, Decode, Ed25519PublicKey, Ed25519Seed, Ed25519Signature, Signature};
use signatory_dalek::{Ed25519Signer, Ed25519Verifier};
#[cfg(feature = "yubihsm")]
use signatory_yubihsm::yubihsm;
use std::io;
use std::marker::{Send, Sync};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time;
use std::{
    ffi::OsStr,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Child, Command},
};
use subtle_encoding::Encoding;
use tempfile::NamedTempFile;
use tm_secret_connection::SecretConnection;
use types::*;
use unix_connection::UNIXConnection;

/// Integration tests for the KMS command-line interface
mod cli;

/// Path to the KMS executable
pub const KMS_EXE_PATH: &str = "./target/debug/tmkms";

mod types {
    include!("../src/types/mod.rs");
}

#[macro_use]
mod error {
    include!("../src/error.rs");
}

mod unix_connection {
    include!("../src/unix_connection.rs");
}

enum KmsSocket {
    /// TCP socket type
    TCP(TcpStream),

    /// UNIX socket type
    UNIX(UnixStream),
}

enum KmsConnection {
    /// Secret connection type
    SecretConnection(SecretConnection<TcpStream>),

    /// UNIX connection type
    UNIXConnection(UNIXConnection<UnixStream>),
}

impl io::Write for KmsConnection {
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        match *self {
            KmsConnection::SecretConnection(ref mut conn) => conn.write(data),
            KmsConnection::UNIXConnection(ref mut conn) => conn.write(data),
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        match *self {
            KmsConnection::SecretConnection(ref mut conn) => conn.flush(),
            KmsConnection::UNIXConnection(ref mut conn) => conn.flush(),
        }
    }
}

impl io::Read for KmsConnection {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, io::Error> {
        match *self {
            KmsConnection::SecretConnection(ref mut conn) => conn.read(data),
            KmsConnection::UNIXConnection(ref mut conn) => conn.read(data),
        }
    }
}

/// Receives incoming KMS connection then sends commands
struct KmsDevice {
    /// KMS child process
    process: Child,

    /// A socket to KMS process
    socket: KmsSocket,
}

impl KmsDevice {
    /// Spawn the KMS process and wait for an incoming TCP connection
    pub fn create_tcp() -> Self {
        // Generate a random port and a config file
        let mut rng = rand::thread_rng();
        let port: u16 = rng.gen_range(60000, 65535);
        let config = KmsDevice::create_tcp_config(port);

        // Listen on a random port
        let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", port)).unwrap();

        let args = &["run", "-c", config.path().to_str().unwrap()];
        let process = Command::new(KMS_EXE_PATH).args(args).spawn().unwrap();

        let (socket, _) = listener.accept().unwrap();
        Self {
            process: process,
            socket: KmsSocket::TCP(socket),
        }
    }

    /// Spawn the KMS process and connect to the Unix listener
    pub fn create_unix() -> Self {
        // Create a random socket path and a config file
        let mut rng = rand::thread_rng();
        let letter: char = rng.gen_range(b'a', b'z') as char;
        let number: u32 = rng.gen_range(0, 999999);
        let socket_path = format!("/tmp/tmkms-{}{:06}.sock", letter, number);
        let config = KmsDevice::create_unix_config(&socket_path);

        // Launch KMS process first to avoid a race condition on the socket path
        let args = &["run", "-c", config.path().to_str().unwrap()];
        let process = Command::new(KMS_EXE_PATH).args(args).spawn().unwrap();

        // TODO(amr): find a better way to wait
        // Sleep for 1s to give the process a chance to create a UnixListener
        thread::sleep(time::Duration::from_millis(100));

        let socket = UnixStream::connect(socket_path).unwrap();
        Self {
            process: process,
            socket: KmsSocket::UNIX(socket),
        }
    }

    /// Create a config file for a TCP KMS and return its path
    fn create_tcp_config(port: u16) -> NamedTempFile {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"
            [[providers.softsign]]
            id = "example-key-1"
            path = "tests/signing.key"

            [[validator]]
            reconnect = false

                [validator.seccon]
                addr = "127.0.0.1"
                port = {}
                secret-key-path = "tests/seccon.key"
        "#,
            port
        );

        config_file
    }

    /// Create a config file for a UNIX KMS and return its path
    fn create_unix_config(socket_path: &str) -> NamedTempFile {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"
            [[providers.softsign]]
            id = "example-key-1"
            path = "tests/signing.key"

            [[validator]]

                [validator.unix]
                    socket-path = "{}"
        "#,
            socket_path
        );

        config_file
    }

    /// Get a connection from the socket
    pub fn create_connection(&self) -> KmsConnection {
        match self.socket {
            KmsSocket::TCP(ref sock) => {
                // we use the same key for both sides:
                let (_, signer) = test_key();

                // Here we reply to the kms with a "remote" ephermal key, auth signature etc:
                let socket_cp = sock.try_clone().unwrap();
                let public_key = signatory::public_key(&signer).unwrap();

                KmsConnection::SecretConnection(
                    SecretConnection::new(socket_cp, &public_key, &signer).unwrap(),
                )
            }

            KmsSocket::UNIX(ref sock) => {
                let socket_cp = sock.try_clone().unwrap();

                KmsConnection::UNIXConnection(UNIXConnection::new(socket_cp).unwrap())
            }
        }
    }
}

/// A struct to hold protocol integration tests contexts
struct ProtocolTester {
    tcp_device: KmsDevice,
    tcp_connection: KmsConnection,
    unix_device: KmsDevice,
    unix_connection: KmsConnection,
}

impl ProtocolTester {
    pub fn apply<F>(functor: F)
    where
        F: FnOnce(ProtocolTester),
    {
        let tcp_device = KmsDevice::create_tcp();
        let tcp_connection = tcp_device.create_connection();
        let unix_device = KmsDevice::create_unix();
        let unix_connection = unix_device.create_connection();

        functor(Self {
            tcp_device,
            tcp_connection,
            unix_device,
            unix_connection,
        });
    }
}

impl Drop for ProtocolTester {
    fn drop(&mut self) {
        self.tcp_device.process.wait().unwrap();
        self.unix_device.process.wait().unwrap();
    }
}

impl io::Write for ProtocolTester {
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        let unix_sz = self.unix_connection.write(data)?;
        let tcp_sz = self.tcp_connection.write(data)?;

        // Assert caller sanity
        assert!(unix_sz == tcp_sz);
        Ok(unix_sz)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.unix_connection.flush()?;
        self.tcp_connection.flush()?;
        Ok(())
    }
}

impl io::Read for ProtocolTester {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, io::Error> {
        let mut unix_buf = vec![0u8; data.len()];

        let tcp_sz = self.tcp_connection.read(data)?;
        let unix_sz = self.unix_connection.read(&mut unix_buf)?;

        // Assert handler sanity
        assert!(
            unix_buf == data,
            "binary protocol differs between TCP and UNIX sockets"
        );

        Ok(unix_sz)
    }
}

/// Get the public key associated with the testing private key
fn test_key() -> (Ed25519PublicKey, Ed25519Signer) {
    let seed =
        ed25519::Seed::decode_from_file("tests/signing.key", subtle_encoding::IDENTITY).unwrap();
    let signer = Ed25519Signer::from(&seed);
    (signatory::public_key(&signer).unwrap(), signer)
}

/// Construct and send a poison pill message to stop KMS devices
fn send_poison_pill(pt: &mut ProtocolTester) {
    let pill = types::PoisonPillMsg {};
    let mut buf = vec![];

    // Use connection to send a message
    pill.encode(&mut buf).unwrap();
    pt.write_all(&buf).unwrap();

    println!("sent poison pill");
}

#[test]
fn test_handle_and_sign_heartbeat() {
    let chain_id = "test_chain_id";
    let (pub_key, _) = test_key();

    ProtocolTester::apply(|mut pt| {
        // prep a request:
        let addr = vec![
            0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
            0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
        ];

        let hb = heartbeat::Heartbeat {
            validator_address: addr,
            validator_index: 1,
            height: 15,
            round: 10,
            sequence: 30,
            signature: None,
        };

        let hb_msg = heartbeat::SignHeartbeatRequest {
            heartbeat: Some(hb),
        };

        // send request:
        let mut buf = vec![];
        hb_msg.encode(&mut buf).unwrap();
        pt.write_all(&buf).unwrap();

        // receive response:
        let mut resp_buf = vec![0u8; 512];
        pt.read(&mut resp_buf).unwrap();

        let actual_len = extract_actual_len(&resp_buf).unwrap();
        let mut resp = vec![0u8; actual_len as usize];
        resp.copy_from_slice(&resp_buf[..actual_len as usize]);

        let hb_req: heartbeat::SignHeartbeatRequest =
            heartbeat::SignHeartbeatRequest::decode(&resp).expect("decoding heartbeat failed");
        let mut sign_bytes: Vec<u8> = vec![];

        hb_req.sign_bytes(chain_id, &mut sign_bytes).unwrap();

        let hb: heartbeat::Heartbeat = hb_req
            .heartbeat
            .expect("heartbeat should be embedded but none was found");
        let sig: Vec<u8> = hb.signature.expect("expected signature was not found");
        let verifier = Ed25519Verifier::from(&pub_key);
        let signature = Ed25519Signature::from_bytes(sig).unwrap();
        let msg: &[u8] = sign_bytes.as_slice();

        ed25519::verify(&verifier, msg, &signature).unwrap();

        send_poison_pill(&mut pt);
    });
}

#[test]
fn test_handle_and_sign_proposal() {
    let chain_id = "test_chain_id";
    let (pub_key, _) = test_key();

    let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
    let t = Time {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    };

    ProtocolTester::apply(|mut pt| {
        let proposal = types::proposal::Proposal {
            height: 12345,
            round: 23456,
            timestamp: Some(t),
            block_parts_header: Some(PartsSetHeader {
                total: 111,
                hash: "blockparts".as_bytes().to_vec(),
            }),
            pol_round: -1,
            pol_block_id: None,
            signature: None,
        };

        let spr = types::proposal::SignProposalRequest {
            proposal: Some(proposal),
        };

        let mut buf = vec![];
        spr.encode(&mut buf).unwrap();
        pt.write_all(&buf).unwrap();

        // receive response:
        let mut resp_buf = vec![0u8; 1024];
        pt.read(&mut resp_buf).unwrap();

        let actual_len = extract_actual_len(&resp_buf).unwrap();
        let mut resp = vec![0u8; actual_len as usize];
        resp.copy_from_slice(&mut resp_buf[..(actual_len as usize)]);

        let p_req =
            proposal::SignedProposalResponse::decode(&resp).expect("decoding proposal failed");
        let mut sign_bytes: Vec<u8> = vec![];
        spr.sign_bytes(chain_id, &mut sign_bytes).unwrap();

        let prop: types::proposal::Proposal = p_req
            .proposal
            .expect("proposal should be embedded but none was found");
        let sig: Vec<u8> = prop.signature.expect("expected signature was not found");
        let verifier = Ed25519Verifier::from(&pub_key);
        let signature = Ed25519Signature::from_bytes(sig).unwrap();
        let msg: &[u8] = sign_bytes.as_slice();

        ed25519::verify(&verifier, msg, &signature).unwrap();

        send_poison_pill(&mut pt);
    });
}

#[test]
fn test_handle_and_sign_vote() {
    let chain_id = "test_chain_id";
    let (pub_key, _) = test_key();

    let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
    let t = Time {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    };

    ProtocolTester::apply(|mut pt| {
        let vote_msg = types::vote::Vote {
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            vote_type: 0x01,
            block_id: Some(BlockID {
                hash: "hash".as_bytes().to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
                }),
            }),
            signature: vec![],
        };

        let svr = types::vote::SignVoteRequest {
            vote: Some(vote_msg),
        };
        let mut buf = vec![];
        svr.encode(&mut buf).unwrap();
        pt.write_all(&buf).unwrap();

        // receive response:
        let mut resp_buf = vec![0u8; 1024];
        pt.read(&mut resp_buf).unwrap();

        let actual_len = extract_actual_len(&resp_buf).unwrap();
        let mut resp = vec![0u8; actual_len as usize];
        resp.copy_from_slice(&resp_buf[..actual_len as usize]);

        let v_resp = vote::SignedVoteResponse::decode(&resp).expect("decoding vote failed");
        let mut sign_bytes: Vec<u8> = vec![];
        svr.sign_bytes(chain_id, &mut sign_bytes).unwrap();

        let vote_msg: types::vote::Vote = v_resp
            .vote
            .expect("vote should be embedded int the response but none was found");
        let sig: Vec<u8> = vote_msg.signature;
        assert_ne!(sig, vec![]);
        let verifier = Ed25519Verifier::from(&pub_key);
        let signature = Ed25519Signature::from_bytes(sig).unwrap();
        let msg: &[u8] = sign_bytes.as_slice();

        ed25519::verify(&verifier, msg, &signature).unwrap();

        send_poison_pill(&mut pt);
    });
}

#[test]
fn test_handle_and_sign_get_publickey() {
    ProtocolTester::apply(|mut pt| {
        let mut buf = vec![];

        PubKeyMsg {
            pub_key_ed25519: vec![],
        }.encode(&mut buf)
        .unwrap();

        pt.write_all(&buf).unwrap();

        // receive response:
        let mut resp_buf = vec![0u8; 1024];
        pt.read(&mut resp_buf).unwrap();

        let actual_len = extract_actual_len(&resp_buf).unwrap();
        let mut resp = vec![0u8; actual_len as usize];
        resp.copy_from_slice(&resp_buf[..actual_len as usize]);

        let pk_resp = PubKeyMsg::decode(&resp).expect("decoding public key failed");
        assert_ne!(pk_resp.pub_key_ed25519, vec![]);
        println!("got public key: {:?}", pk_resp.pub_key_ed25519);

        send_poison_pill(&mut pt);
    });
}

#[test]
fn test_handle_and_sign_ping_pong() {
    ProtocolTester::apply(|mut pt| {
        let mut buf = vec![];
        PingRequest {}.encode(&mut buf).unwrap();
        pt.write_all(&buf).unwrap();

        // receive response:
        let mut resp_buf = vec![0u8; 1024];
        pt.read(&mut resp_buf).unwrap();

        let actual_len = extract_actual_len(&resp_buf).unwrap();
        let mut resp = vec![0u8; actual_len as usize];
        resp.copy_from_slice(&resp_buf[..actual_len as usize]);
        let pong = PingResponse::decode(&resp).expect("decoding ping response failed");

        send_poison_pill(&mut pt);
    });
}
