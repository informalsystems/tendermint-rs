use std::io::Read as _;
use std::io::Write as _;
use std::thread;

use ed25519_dalek::{self as ed25519};
use eyre::Result;
use rand_core::OsRng;
use x25519_dalek::PublicKey as EphemeralPublic;

use tendermint_p2p::secret_connection::{sort32, Handshake, SecretConnection, Version};
use tendermint_proto as proto;

use crate::pipe;

mod nonce;
mod public_key;

#[test]
fn test_handshake() {
    let (pipe1, pipe2) = pipe::async_bipipe_buffered();

    let peer1 = thread::spawn(|| {
        let conn1 = new_peer_conn(pipe2);
        assert!(conn1.is_ok());
    });

    let peer2 = thread::spawn(|| {
        let conn2 = new_peer_conn(pipe1);
        assert!(conn2.is_ok());
    });

    peer1.join().expect("peer1 thread has panicked");
    peer2.join().expect("peer2 thread has panicked");
}

#[test]
fn test_read_write_single_message() {
    const MESSAGE: &str = "The Queen's Gambit";

    let (pipe1, pipe2) = pipe::async_bipipe_buffered();

    let sender = thread::spawn(move || {
        let mut conn1 = new_peer_conn(pipe2).expect("handshake to succeed");

        conn1
            .write_all(MESSAGE.as_bytes())
            .expect("expected to write message");
    });

    let receiver = thread::spawn(move || {
        let mut conn2 = new_peer_conn(pipe1).expect("handshake to succeed");

        let mut buf = [0; MESSAGE.len()];
        conn2
            .read_exact(&mut buf)
            .expect("expected to read message");
        assert_eq!(MESSAGE.as_bytes(), &buf);
    });

    sender.join().expect("sender thread has panicked");
    receiver.join().expect("receiver thread has panicked");
}

#[test]
fn test_evil_peer_shares_invalid_eph_key() {
    let mut csprng = OsRng {};
    let local_privkey: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
    let (mut h, _) = Handshake::new(local_privkey, Version::V0_34);
    let bytes: [u8; 32] = [0; 32];
    let res = h.got_key(EphemeralPublic::from(bytes));
    assert!(res.is_err());
}

#[test]
fn test_evil_peer_shares_invalid_auth_sig() {
    let mut csprng = OsRng {};
    let local_privkey: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
    let (mut h, _) = Handshake::new(local_privkey, Version::V0_34);
    let res = h.got_key(EphemeralPublic::from(x25519_dalek::X25519_BASEPOINT_BYTES));
    assert!(res.is_ok());

    let mut h = res.unwrap();
    let res = h.got_signature(proto::p2p::AuthSigMessage {
        pub_key: None,
        sig: vec![],
    });
    assert!(res.is_err());
}

#[test]
fn test_sort() {
    // sanity check
    let t1 = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let t2 = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ];
    let (ref t3, ref t4) = sort32(t1, t2);
    assert_eq!(t1, *t3);
    assert_eq!(t2, *t4);
}

fn new_peer_conn<IoHandler>(io_handler: IoHandler) -> Result<SecretConnection<IoHandler>>
where
    IoHandler: std::io::Read + std::io::Write + Send + Sync,
{
    let mut csprng = OsRng {};
    let privkey1: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
    SecretConnection::new(io_handler, privkey1, Version::V0_34)
}
