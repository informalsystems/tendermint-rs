use std::io::Read as _;
use std::io::Write as _;
use std::thread;

use ed25519_dalek::{self as ed25519};
use rand_core::OsRng;
use x25519_dalek::PublicKey as EphemeralPublic;

use tendermint_p2p::secret_connection::{sort32, Handshake, SecretConnection, Version};
use tendermint_proto as proto;

use crate::pipe;

mod public_key;

#[test]
fn test_handshake() {
    let (pipe1, pipe2) = pipe::async_bipipe_buffered();

    let peer1 = thread::spawn(|| {
        let mut csprng = OsRng {};
        let privkey1: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
        let conn1 = SecretConnection::new(pipe2, privkey1, Version::V0_34);
        assert_eq!(conn1.is_ok(), true);
    });

    let peer2 = thread::spawn(|| {
        let mut csprng = OsRng {};
        let privkey2: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
        let conn2 = SecretConnection::new(pipe1, privkey2, Version::V0_34);
        assert_eq!(conn2.is_ok(), true);
    });

    peer1.join().expect("peer1 thread has panicked");
    peer2.join().expect("peer2 thread has panicked");
}

#[test]
fn test_read_write_single_message() {
    const MESSAGE: &str = "The Queen's Gambit";

    let (pipe1, pipe2) = pipe::async_bipipe_buffered();

    let sender = thread::spawn(move || {
        let mut csprng = OsRng {};
        let privkey1: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
        let mut conn1 =
            SecretConnection::new(pipe2, privkey1, Version::V0_34).expect("handshake to succeed");

        conn1
            .write_all(MESSAGE.as_bytes())
            .expect("expected to write message");
    });

    let receiver = thread::spawn(move || {
        let mut csprng = OsRng {};
        let privkey2: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
        let mut conn2 =
            SecretConnection::new(pipe1, privkey2, Version::V0_34).expect("handshake to succeed");

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
    assert_eq!(res.is_err(), true);
}

#[test]
fn test_evil_peer_shares_invalid_auth_sig() {
    let mut csprng = OsRng {};
    let local_privkey: ed25519::Keypair = ed25519::Keypair::generate(&mut csprng);
    let (mut h, _) = Handshake::new(local_privkey, Version::V0_34);
    let res = h.got_key(EphemeralPublic::from(x25519_dalek::X25519_BASEPOINT_BYTES));
    assert_eq!(res.is_err(), false);

    let mut h = res.unwrap();
    let res = h.got_signature(proto::p2p::AuthSigMessage {
        pub_key: None,
        sig: vec![],
    });
    assert_eq!(res.is_err(), true);
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
