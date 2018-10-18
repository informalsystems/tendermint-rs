//! KMS integration test

// TODO: get rid of hacks for using RPC types in tests
#![allow(unused_imports, unused_variables, dead_code)]

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

/// Hacks for accessing the RPC types in tests
#[macro_use]
extern crate serde_json;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate failure;
extern crate hkdf;
extern crate ring;
extern crate sha2;
extern crate x25519_dalek;

use prost::Message;
use signatory::{ed25519, encoding::Decode, Signer};
use signatory_dalek::Ed25519Signer;
#[cfg(feature = "yubihsm")]
use signatory_yubihsm::yubihsm;
use std::{
    ffi::OsStr,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Child, Command},
};
use std::io;
use std::time;
use std::thread;
use std::marker::{Send, Sync};
use std::os::unix::net::UnixStream;
use subtle_encoding::Encoding;
use types::TendermintSign;
use tempfile::NamedTempFile;
use rand::Rng;

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

mod secret_connection {
    include!("../src/secret_connection.rs");
}

mod unix_connection {
    include!("../src/unix_connection.rs");
}

use secret_connection::SecretConnection;
use unix_connection::UNIXConnection;

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
        let listener =
            TcpListener::bind(format!("{}:{}", "127.0.0.1", port)).unwrap();

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
        writeln!(config_file, r#"
            [[providers.softsign]]
            id = "example-key-1"
            path = "tests/signing.key"

            [[validator]]
            reconnect = false

                [validator.seccon]
                addr = "127.0.0.1"
                port = {}
                secret-key-path = "tests/seccon.key"
        "#, port);

        config_file
    }

    /// Create a config file for a UNIX KMS and return its path
    fn create_unix_config(socket_path: &str) -> NamedTempFile {
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, r#"
            [[providers.softsign]]
            id = "example-key-1"
            path = "tests/signing.key"

            [[validator]]

                [validator.unix]
                    socket-path = "{}"
        "#, socket_path);

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
                    SecretConnection::new(socket_cp, &public_key, &signer).unwrap())
            },

            KmsSocket::UNIX(ref sock) => {
                let socket_cp = sock.try_clone().unwrap();

                KmsConnection::UNIXConnection(
                    UNIXConnection::new(socket_cp).unwrap())
            }
        }
    }

    /// Sign the given message with the given public key using the KMS
    pub fn sign(
        &mut self,
        public_key: &ed25519::PublicKey,
        signer: &Signer<ed25519::Signature>,
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
        signatory::sign(signer, &json_msg.into_bytes()).unwrap()
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
        where F: FnOnce(ProtocolTester)
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

/// Get the public key associated with the testing private key
fn test_key() -> (ed25519::PublicKey, Ed25519Signer) {
    let seed =
        ed25519::Seed::decode_from_file("tests/signing.key", subtle_encoding::IDENTITY).unwrap();
    let signer = Ed25519Signer::from(&seed);
    (signatory::public_key(&signer).unwrap(), signer)
}

#[test]
fn test_handle_poisonpill() {
    ProtocolTester::apply(|mut pt| {
        let pill = types::PoisonPillMsg {};
        let mut buf = vec![];

        // Use connection to send a message
        pill.encode(&mut buf).unwrap();
        pt.write_all(&buf).unwrap();

        println!("sent poison pill");
    });
}
