//! KMS integration test

extern crate prost_amino as prost;

use crate::prost::Message;
use chrono::{DateTime, Utc};
use rand::Rng;
use signatory::{ed25519, encoding::Identity, Decode, Signature};
use signatory_dalek::{Ed25519Signer, Ed25519Verifier};
use std::{
    fs,
    io::{self, Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::{UnixListener, UnixStream},
    process::{Child, Command},
};
use tempfile::NamedTempFile;
use tendermint::{
    amino_types::{self, *},
    secret_connection::{self, SecretConnection},
};
use tmkms::UnixConnection;

/// Integration tests for the KMS command-line interface
mod cli;

/// Path to the KMS executable
const KMS_EXE_PATH: &str = "target/debug/tmkms";

/// Path to the example validator signing key
const SIGNING_KEY_PATH: &str = "tests/support/signing.key";

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
    UNIXConnection(UnixConnection<UnixStream>),
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
struct KmsProcess {
    /// KMS child process
    process: Child,

    /// A socket to KMS process
    socket: KmsSocket,
}

impl KmsProcess {
    /// Spawn the KMS process and wait for an incoming TCP connection
    pub fn create_tcp() -> Self {
        // Generate a random port and a config file
        let port: u16 = rand::thread_rng().gen_range(60000, 65535);
        let config = KmsProcess::create_tcp_config(port);

        // Listen on a random port
        let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", port)).unwrap();

        let args = &["start", "-c", config.path().to_str().unwrap()];
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
        let config = KmsProcess::create_unix_config(&socket_path);

        // Start listening for connections via the Unix socket
        let listener = UnixListener::bind(socket_path).unwrap();

        // Fire up the KMS process and allow it to connect to our Unix socket
        let args = &["start", "-c", config.path().to_str().unwrap()];
        let process = Command::new(KMS_EXE_PATH).args(args).spawn().unwrap();

        let (socket, _) = listener.accept().unwrap();
        Self {
            process: process,
            socket: KmsSocket::UNIX(socket),
        }
    }

    /// Create a config file for a TCP KMS and return its path
    fn create_tcp_config(port: u16) -> NamedTempFile {
        let mut config_file = NamedTempFile::new().unwrap();
        let peer_id = "f88883b673fc69d7869cab098de3bafc2ff76eb8";

        writeln!(
            config_file,
            r#"
            [[chain]]
            id = "test_chain_id"
            key_format = {{ type = "bech32", account_key_prefix = "cosmospub", consensus_key_prefix = "cosmosvalconspub" }}

            [[validator]]
            addr = "tcp://{}@127.0.0.1:{}"
            chain_id = "test_chain_id"
            max_height = "500000"
            reconnect = false
            secret_key = "tests/support/secret_connection.key"

            [[providers.softsign]]
            chain_ids = ["test_chain_id"]
            path = "{}"
        "#,
            peer_id, port, SIGNING_KEY_PATH
        )
        .unwrap();

        config_file
    }

    /// Create a config file for a UNIX KMS and return its path
    fn create_unix_config(socket_path: &str) -> NamedTempFile {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"
            [[chain]]
            id = "test_chain_id"
            key_format = {{ type = "bech32", account_key_prefix = "cosmospub", consensus_key_prefix = "cosmosvalconspub" }}

            [[validator]]
            addr = "unix://{}"
            chain_id = "test_chain_id"
            max_height = "500000"


            [[providers.softsign]]
            chain_ids = ["test_chain_id"]
            path = "{}"
        "#,
            socket_path, SIGNING_KEY_PATH
        )
        .unwrap();

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
                let public_key =
                    secret_connection::PublicKey::from(signatory::public_key(&signer).unwrap());

                KmsConnection::SecretConnection(
                    SecretConnection::new(socket_cp, &public_key, &signer).unwrap(),
                )
            }

            KmsSocket::UNIX(ref sock) => {
                let socket_cp = sock.try_clone().unwrap();

                KmsConnection::UNIXConnection(UnixConnection::new(socket_cp))
            }
        }
    }
}

/// A struct to hold protocol integration tests contexts
struct ProtocolTester {
    tcp_device: KmsProcess,
    tcp_connection: KmsConnection,
    unix_device: KmsProcess,
    unix_connection: KmsConnection,
}

impl ProtocolTester {
    pub fn apply<F>(functor: F)
    where
        F: FnOnce(ProtocolTester),
    {
        let tcp_device = KmsProcess::create_tcp();
        let tcp_connection = tcp_device.create_connection();
        let unix_device = KmsProcess::create_unix();
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
        self.tcp_device.process.kill().unwrap();
        self.unix_device.process.kill().unwrap();

        match fs::remove_file("test_chain_id_priv_validator_state.json") {
            Err(ref e) if e.kind() != io::ErrorKind::NotFound => {
                panic!("{}", e);
            }
            _ => (),
        }
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

        self.tcp_connection.read(data)?;
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
fn test_key() -> (ed25519::PublicKey, Ed25519Signer) {
    let seed = ed25519::Seed::decode_from_file(SIGNING_KEY_PATH, &Identity::default()).unwrap();
    let signer = Ed25519Signer::from(&seed);
    (signatory::public_key(&signer).unwrap(), signer)
}

/// Extract the actual length of an amino message
pub fn extract_actual_len(buf: &[u8]) -> Result<u64, prost::DecodeError> {
    let mut buff = Cursor::new(buf);
    let actual_len = prost::encoding::decode_varint(&mut buff)?;
    if actual_len == 0 {
        return Ok(1);
    }
    Ok(actual_len + (prost::encoding::encoded_len_varint(actual_len) as u64))
}

#[test]
fn test_handle_and_sign_proposal() {
    let chain_id = "test_chain_id";
    let (pub_key, _) = test_key();

    let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
    let t = TimeMsg {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    };

    ProtocolTester::apply(|mut pt| {
        let proposal = amino_types::proposal::Proposal {
            msg_type: amino_types::SignedMsgType::Proposal.to_u32(),
            height: 12345,
            round: 1,
            timestamp: Some(t),
            pol_round: -1,
            block_id: None,
            signature: vec![],
        };

        let spr = amino_types::proposal::SignProposalRequest {
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
        spr.sign_bytes(chain_id.into(), &mut sign_bytes).unwrap();

        let prop: amino_types::proposal::Proposal = p_req
            .proposal
            .expect("proposal should be embedded but none was found");
        let verifier = Ed25519Verifier::from(&pub_key);
        let signature = ed25519::Signature::from_bytes(prop.signature).unwrap();
        let msg: &[u8] = sign_bytes.as_slice();

        ed25519::verify(&verifier, msg, &signature).unwrap();
    });
}

#[test]
fn test_handle_and_sign_vote() {
    let chain_id = "test_chain_id";
    let (pub_key, _) = test_key();

    let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
    let t = TimeMsg {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    };

    ProtocolTester::apply(|mut pt| {
        let vote_msg = amino_types::vote::Vote {
            vote_type: 0x01,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            block_id: Some(BlockId {
                hash: b"some hash00000000000000000000000".to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: b"parts_hash0000000000000000000000".to_vec(),
                }),
            }),
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            signature: vec![],
        };

        let svr = amino_types::vote::SignVoteRequest {
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
        svr.sign_bytes(chain_id.into(), &mut sign_bytes).unwrap();

        let vote_msg: amino_types::vote::Vote = v_resp
            .vote
            .expect("vote should be embedded int the response but none was found");

        let sig: Vec<u8> = vote_msg.signature;
        assert_ne!(sig.len(), 0);

        let verifier = Ed25519Verifier::from(&pub_key);
        let signature = ed25519::Signature::from_bytes(sig).unwrap();
        let msg: &[u8] = sign_bytes.as_slice();

        ed25519::verify(&verifier, msg, &signature).unwrap();
    });
}

#[test]
#[should_panic]
fn test_exceed_max_height() {
    let chain_id = "test_chain_id";
    let (pub_key, _) = test_key();

    let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
    let t = TimeMsg {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    };

    ProtocolTester::apply(|mut pt| {
        let vote_msg = amino_types::vote::Vote {
            vote_type: 0x01,
            height: 500001,
            round: 2,
            timestamp: Some(t),
            block_id: Some(BlockId {
                hash: b"some hash00000000000000000000000".to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: b"parts_hash0000000000000000000000".to_vec(),
                }),
            }),
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            signature: vec![],
        };

        let svr = amino_types::vote::SignVoteRequest {
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
        svr.sign_bytes(chain_id.into(), &mut sign_bytes).unwrap();

        let vote_msg: amino_types::vote::Vote = v_resp
            .vote
            .expect("vote should be embedded int the response but none was found");

        let sig: Vec<u8> = vote_msg.signature;
        assert_ne!(sig.len(), 0);

        let verifier = Ed25519Verifier::from(&pub_key);
        let signature = ed25519::Signature::from_bytes(sig).unwrap();
        let msg: &[u8] = sign_bytes.as_slice();

        ed25519::verify(&verifier, msg, &signature).unwrap();
    });
}

#[test]
fn test_handle_and_sign_get_publickey() {
    ProtocolTester::apply(|mut pt| {
        let mut buf = vec![];

        PubKeyRequest {}.encode(&mut buf).unwrap();

        pt.write_all(&buf).unwrap();

        // receive response:
        let mut resp_buf = vec![0u8; 1024];
        pt.read(&mut resp_buf).unwrap();

        let actual_len = extract_actual_len(&resp_buf).unwrap();
        let mut resp = vec![0u8; actual_len as usize];
        resp.copy_from_slice(&resp_buf[..actual_len as usize]);

        let pk_resp = PubKeyResponse::decode(&resp).expect("decoding public key failed");
        assert_ne!(pk_resp.pub_key_ed25519.len(), 0);
        println!("got public key: {:?}", pk_resp.pub_key_ed25519);
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
        PingResponse::decode(&resp).expect("decoding ping response failed");
    });
}
