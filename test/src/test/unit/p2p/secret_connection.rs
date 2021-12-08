use std::io::Read as _;
use std::io::Write as _;
use std::net::{TcpListener, TcpStream};
use std::thread;

use ed25519_consensus;
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
    let local_privkey = ed25519_consensus::SigningKey::new(&mut csprng);
    let (mut h, _) = Handshake::new(local_privkey, Version::V0_34);
    let bytes: [u8; 32] = [0; 32];
    let res = h.got_key(EphemeralPublic::from(bytes));
    assert!(res.is_err());
}

#[test]
fn test_evil_peer_shares_invalid_auth_sig() {
    let mut csprng = OsRng {};
    let local_privkey = ed25519_consensus::SigningKey::new(&mut csprng);
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

#[test]
fn test_split_secret_connection() {
    const MESSAGES_1_TO_2: &[&str] = &["one", "three", "five", "seven"];
    const MESSAGES_2_TO_1: &[&str] = &["two", "four", "six", "eight"];
    let peer1_listener = TcpListener::bind("127.0.0.1:0").expect("to be able to bind to 127.0.0.1");
    let peer1_addr = peer1_listener.local_addr().unwrap();
    println!("peer1 bound to {:?}", peer1_addr);

    let peer1 = thread::spawn(move || {
        let stream = peer1_listener
            .incoming()
            .next()
            .unwrap()
            .expect("an incoming TCP stream from peer 2");
        let mut conn_to_peer2 = new_peer_conn(stream).expect("handshake to succeed");
        println!("peer1 handshake concluded");
        for msg_counter in 0..MESSAGES_1_TO_2.len() {
            // Peer 1 sends first
            conn_to_peer2
                .write_all(MESSAGES_1_TO_2[msg_counter].as_bytes())
                .expect("to write message successfully to peer 2");
            // Peer 1 expects a response
            let mut buf = [0u8; 10];
            let br = conn_to_peer2
                .read(&mut buf)
                .expect("to read a message from peer 2");
            let msg = String::from_utf8_lossy(&buf[0..br]).to_string();
            println!("Got message from peer2: {}", msg);
            assert_eq!(msg, MESSAGES_2_TO_1[msg_counter]);
        }
    });

    // Peer 2 attempts to initiate the secret connection to peer 1
    let peer2_to_peer1 = TcpStream::connect(peer1_addr).expect("to be able to connect to peer 1");
    println!("peer2 connected to peer1");
    let conn_to_peer1 = new_peer_conn(peer2_to_peer1).expect("handshake to succeed");
    println!("peer2 handshake concluded");

    let (mut write_conn, mut read_conn) = conn_to_peer1
        .split()
        .expect("to be able to clone the underlying TcpStream");
    let (write_tx, write_rx) = std::sync::mpsc::channel::<String>();

    // We spawn a standalone thread that makes use of peer2's secret connection
    // purely to write outgoing messages.
    let peer2_writer = thread::spawn(move || {
        for _ in 0..MESSAGES_2_TO_1.len() {
            let msg = write_rx
                .recv()
                .expect("to successfully receive a message to be sent to peer1");
            write_conn
                .write_all(msg.as_bytes())
                .expect("to be able to write to peer 1");
        }
    });

    for msg_counter in 0..MESSAGES_2_TO_1.len() {
        // Wait for peer 1 to send first
        let mut buf = [0u8; 10];
        let br = read_conn
            .read(&mut buf)
            .expect("to receive a message from peer 1");
        let msg = String::from_utf8_lossy(&buf[0..br]).to_string();
        println!("Got message from peer1: {}", msg);
        assert_eq!(msg, MESSAGES_1_TO_2[msg_counter]);
        write_tx
            .send(MESSAGES_2_TO_1[msg_counter].to_string())
            .expect("to be able to communicate with peer2's writer thread");
    }

    peer2_writer
        .join()
        .expect("peer 2's writer thread to run to completion");
    peer1.join().expect("peer 1's thread to run to completion")
}

fn new_peer_conn<IoHandler>(
    io_handler: IoHandler,
) -> Result<SecretConnection<IoHandler>, tendermint_p2p::error::Error>
where
    IoHandler: std::io::Read + std::io::Write + Send + Sync,
{
    let mut csprng = OsRng {};
    let privkey1 = ed25519_consensus::SigningKey::new(&mut csprng);
    SecretConnection::new(io_handler, privkey1, Version::V0_34)
}
