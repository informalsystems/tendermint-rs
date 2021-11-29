//! `SecretConnection`: Transport layer encryption for Tendermint P2P connections.

use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::io::{self, Read, Write};
use std::marker::{Send, Sync};
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::Error;
use chacha20poly1305::{
    aead::{generic_array::GenericArray, AeadInPlace, NewAead},
    ChaCha20Poly1305,
};
use ed25519_dalek::{self as ed25519, Signer, Verifier};
use merlin::Transcript;
use rand_core::OsRng;
use subtle::ConstantTimeEq;
use x25519_dalek::{EphemeralSecret, PublicKey as EphemeralPublic};

use tendermint_proto as proto;
use tendermint_std_ext::TryClone;

pub use self::{
    kdf::Kdf,
    nonce::{Nonce, SIZE as NONCE_SIZE},
    protocol::Version,
    public_key::PublicKey,
};

#[cfg(feature = "amino")]
mod amino_types;

mod kdf;
mod nonce;
mod protocol;
mod public_key;

/// Size of the MAC tag
pub const TAG_SIZE: usize = 16;

/// Maximum size of a message
pub const DATA_MAX_SIZE: usize = 1024;

/// 4 + 1024 == 1028 total frame size
const DATA_LEN_SIZE: usize = 4;
const TOTAL_FRAME_SIZE: usize = DATA_MAX_SIZE + DATA_LEN_SIZE;

/// Handshake is a process of establishing the `SecretConnection` between two peers.
/// [Specification](https://github.com/tendermint/spec/blob/master/spec/p2p/peer.md#authenticated-encryption-handshake)
pub struct Handshake<S> {
    protocol_version: Version,
    state: S,
}

/// Handshake states

/// `AwaitingEphKey` means we're waiting for the remote ephemeral pubkey.
pub struct AwaitingEphKey {
    local_privkey: ed25519::Keypair,
    local_eph_privkey: Option<EphemeralSecret>,
}

/// `AwaitingAuthSig` means we're waiting for the remote authenticated signature.
pub struct AwaitingAuthSig {
    sc_mac: [u8; 32],
    kdf: Kdf,
    recv_cipher: ChaCha20Poly1305,
    send_cipher: ChaCha20Poly1305,
    local_signature: ed25519::Signature,
}

#[allow(clippy::use_self)]
impl Handshake<AwaitingEphKey> {
    /// Initiate a handshake.
    #[must_use]
    pub fn new(
        local_privkey: ed25519::Keypair,
        protocol_version: Version,
    ) -> (Self, EphemeralPublic) {
        // Generate an ephemeral key for perfect forward secrecy.
        let local_eph_privkey = EphemeralSecret::new(&mut OsRng);
        let local_eph_pubkey = EphemeralPublic::from(&local_eph_privkey);

        (
            Self {
                protocol_version,
                state: AwaitingEphKey {
                    local_privkey,
                    local_eph_privkey: Some(local_eph_privkey),
                },
            },
            local_eph_pubkey,
        )
    }

    /// Performs a Diffie-Hellman key agreement and creates a local signature.
    /// Transitions Handshake into `AwaitingAuthSig` state.
    ///
    /// # Errors
    ///
    /// * if protocol order was violated, e.g. handshake missing
    /// * if challenge signing fails
    pub fn got_key(
        &mut self,
        remote_eph_pubkey: EphemeralPublic,
    ) -> Result<Handshake<AwaitingAuthSig>, Error> {
        let local_eph_privkey = match self.state.local_eph_privkey.take() {
            Some(key) => key,
            None => return Err(Error::missing_secret()),
        };
        let local_eph_pubkey = EphemeralPublic::from(&local_eph_privkey);

        // Compute common shared secret.
        let shared_secret = EphemeralSecret::diffie_hellman(local_eph_privkey, &remote_eph_pubkey);

        let mut transcript = Transcript::new(b"TENDERMINT_SECRET_CONNECTION_TRANSCRIPT_HASH");

        // Reject all-zero outputs from X25519 (i.e. from low-order points)
        //
        // See the following for information on potential attacks this check
        // aids in mitigating:
        //
        // - https://github.com/tendermint/kms/issues/142
        // - https://eprint.iacr.org/2019/526.pdf
        if shared_secret.as_bytes().ct_eq(&[0x00; 32]).unwrap_u8() == 1 {
            return Err(Error::low_order_key());
        }

        // Sort by lexical order.
        let local_eph_pubkey_bytes = *local_eph_pubkey.as_bytes();
        let (low_eph_pubkey_bytes, high_eph_pubkey_bytes) =
            sort32(local_eph_pubkey_bytes, *remote_eph_pubkey.as_bytes());

        transcript.append_message(b"EPHEMERAL_LOWER_PUBLIC_KEY", &low_eph_pubkey_bytes);
        transcript.append_message(b"EPHEMERAL_UPPER_PUBLIC_KEY", &high_eph_pubkey_bytes);
        transcript.append_message(b"DH_SECRET", shared_secret.as_bytes());

        // Check if the local ephemeral public key was the least, lexicographically sorted.
        let loc_is_least = local_eph_pubkey_bytes == low_eph_pubkey_bytes;

        let kdf = Kdf::derive_secrets_and_challenge(shared_secret.as_bytes(), loc_is_least);

        let mut sc_mac: [u8; 32] = [0; 32];

        transcript.challenge_bytes(b"SECRET_CONNECTION_MAC", &mut sc_mac);

        // Sign the challenge bytes for authentication.
        let local_signature = if self.protocol_version.has_transcript() {
            sign_challenge(&sc_mac, &self.state.local_privkey)?
        } else {
            sign_challenge(&kdf.challenge, &self.state.local_privkey)?
        };

        Ok(Handshake {
            protocol_version: self.protocol_version,
            state: AwaitingAuthSig {
                sc_mac,
                recv_cipher: ChaCha20Poly1305::new(&kdf.recv_secret.into()),
                send_cipher: ChaCha20Poly1305::new(&kdf.send_secret.into()),
                kdf,
                local_signature,
            },
        })
    }
}

impl Handshake<AwaitingAuthSig> {
    /// Returns a verified pubkey of the remote peer.
    ///
    /// # Errors
    ///
    /// * if signature scheme isn't supported
    pub fn got_signature(
        &mut self,
        auth_sig_msg: proto::p2p::AuthSigMessage,
    ) -> Result<PublicKey, Error> {
        let pk_sum = auth_sig_msg
            .pub_key
            .and_then(|key| key.sum)
            .ok_or_else(Error::missing_key)?;

        let remote_pubkey = match pk_sum {
            proto::crypto::public_key::Sum::Ed25519(ref bytes) => {
                ed25519::PublicKey::from_bytes(bytes).map_err(Error::signature)
            }
            _ => Err(Error::unsupported_key()),
        }?;

        let remote_sig =
            ed25519::Signature::try_from(auth_sig_msg.sig.as_slice()).map_err(Error::signature)?;

        if self.protocol_version.has_transcript() {
            remote_pubkey
                .verify(&self.state.sc_mac, &remote_sig)
                .map_err(Error::signature)?;
        } else {
            remote_pubkey
                .verify(&self.state.kdf.challenge, &remote_sig)
                .map_err(Error::signature)?;
        }

        // We've authorized.
        Ok(remote_pubkey.into())
    }
}

// Macro usage allows us to avoid unnecessarily cloning the Arc<AtomicBool>
// that indicates whether we need to terminate the connection.
//
// Limitation: this only checks once prior to the execution of an I/O operation
// whether we need to terminate. This should be sufficient for our purposes
// though.
macro_rules! checked_io {
    ($term:expr, $f:expr) => {{
        if $term.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "secret connection was terminated elsewhere by previous error",
            ));
        }
        let result = { $f };
        if result.is_err() {
            $term.store(true, Ordering::SeqCst);
        }
        result
    }};
}

/// Encrypted connection between peers in a Tendermint network.
///
/// ## Connection integrity and failures
///
/// Due to the underlying encryption mechanism (currently [RFC 8439]), when a
/// read or write failure occurs, it is necessary to disconnect from the remote
/// peer and attempt to reconnect.
///
/// ## Half- and full-duplex connections
/// By default, a `SecretConnection` facilitates half-duplex operations (i.e.
/// one can either read from the connection or write to it at a given time, but
/// not both simultaneously).
///
/// If, however, the underlying I/O handler class implements
/// [`tendermint_std_ext::TryClone`], then you can use
/// [`SecretConnection::split`] to split the `SecretConnection` into its
/// sending and receiving halves. Each of these halves can then be used in a
/// separate thread to facilitate full-duplex communication.
///
/// ## Contracts
///
/// When reading data, data smaller than [`DATA_MAX_SIZE`] is read atomically.
///
/// [RFC 8439]: https://www.rfc-editor.org/rfc/rfc8439.html
pub struct SecretConnection<IoHandler> {
    io_handler: IoHandler,
    protocol_version: Version,
    remote_pubkey: Option<PublicKey>,
    send_state: SendState,
    recv_state: ReceiveState,
    terminate: Arc<AtomicBool>,
}

impl<IoHandler: Read + Write + Send + Sync> SecretConnection<IoHandler> {
    /// Returns the remote pubkey. Panics if there's no key.
    pub fn remote_pubkey(&self) -> PublicKey {
        self.remote_pubkey.expect("remote_pubkey uninitialized")
    }

    /// Performs a handshake and returns a new `SecretConnection`.
    ///
    /// # Errors
    ///
    /// * if sharing of the pubkey fails
    /// * if sharing of the signature fails
    /// * if receiving the signature fails
    pub fn new(
        mut io_handler: IoHandler,
        local_privkey: ed25519::Keypair,
        protocol_version: Version,
    ) -> Result<Self, Error> {
        // Start a handshake process.
        let local_pubkey = PublicKey::from(&local_privkey);
        let (mut h, local_eph_pubkey) = Handshake::new(local_privkey, protocol_version);

        // Write local ephemeral pubkey and receive one too.
        let remote_eph_pubkey =
            share_eph_pubkey(&mut io_handler, &local_eph_pubkey, protocol_version)?;

        // Compute a local signature (also recv_cipher & send_cipher)
        let mut h = h.got_key(remote_eph_pubkey)?;

        let mut sc = Self {
            io_handler,
            protocol_version,
            remote_pubkey: None,
            send_state: SendState {
                cipher: h.state.send_cipher.clone(),
                nonce: Nonce::default(),
            },
            recv_state: ReceiveState {
                cipher: h.state.recv_cipher.clone(),
                nonce: Nonce::default(),
                buffer: vec![],
            },
            terminate: Arc::new(AtomicBool::new(false)),
        };

        // Share each other's pubkey & challenge signature.
        // NOTE: the data must be encrypted/decrypted using ciphers.
        let auth_sig_msg = match local_pubkey {
            PublicKey::Ed25519(ref pk) => {
                share_auth_signature(&mut sc, pk, &h.state.local_signature)?
            }
        };

        // Authenticate remote pubkey.
        let remote_pubkey = h.got_signature(auth_sig_msg)?;

        // All good!
        sc.remote_pubkey = Some(remote_pubkey);
        Ok(sc)
    }
}

impl<IoHandler> SecretConnection<IoHandler>
where
    IoHandler: TryClone,
    <IoHandler as TryClone>::Error: std::error::Error + Send + Sync + 'static,
{
    /// For secret connections whose underlying I/O layer implements
    /// [`tendermint_std_ext::TryClone`], this attempts to split such a
    /// connection into its sending and receiving halves.
    ///
    /// This facilitates full-duplex communications when each half is used in
    /// a separate thread.
    ///
    /// ## Errors
    /// Fails when the `try_clone` operation for the underlying I/O handler
    /// fails.
    pub fn split(self) -> Result<(Sender<IoHandler>, Receiver<IoHandler>), Error> {
        let remote_pubkey = self.remote_pubkey.expect("remote_pubkey to be initialized");
        Ok((
            Sender {
                io_handler: self
                    .io_handler
                    .try_clone()
                    .map_err(|e| Error::transport_clone(e.to_string()))?,
                remote_pubkey,
                state: self.send_state,
                terminate: self.terminate.clone(),
            },
            Receiver {
                io_handler: self.io_handler,
                remote_pubkey,
                state: self.recv_state,
                terminate: self.terminate,
            },
        ))
    }
}

impl<IoHandler: Read> Read for SecretConnection<IoHandler> {
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
        checked_io!(
            self.terminate,
            read_and_decrypt(&mut self.io_handler, &mut self.recv_state, data)
        )
    }
}

impl<IoHandler: Write> Write for SecretConnection<IoHandler> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        checked_io!(
            self.terminate,
            encrypt_and_write(&mut self.io_handler, &mut self.send_state, data)
        )
    }

    fn flush(&mut self) -> io::Result<()> {
        checked_io!(self.terminate, self.io_handler.flush())
    }
}

// Sending state for a `SecretConnection`.
struct SendState {
    cipher: ChaCha20Poly1305,
    nonce: Nonce,
}

// Receiving state for a `SecretConnection`.
struct ReceiveState {
    cipher: ChaCha20Poly1305,
    nonce: Nonce,
    buffer: Vec<u8>,
}

/// The sending end of a [`SecretConnection`].
pub struct Sender<IoHandler> {
    io_handler: IoHandler,
    remote_pubkey: PublicKey,
    state: SendState,
    terminate: Arc<AtomicBool>,
}

impl<IoHandler> Sender<IoHandler> {
    /// Returns the remote pubkey. Panics if there's no key.
    pub const fn remote_pubkey(&self) -> PublicKey {
        self.remote_pubkey
    }
}

impl<IoHandler: Write> Write for Sender<IoHandler> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        checked_io!(
            self.terminate,
            encrypt_and_write(&mut self.io_handler, &mut self.state, buf)
        )
    }

    fn flush(&mut self) -> io::Result<()> {
        checked_io!(self.terminate, self.io_handler.flush())
    }
}

/// The receiving end of a [`SecretConnection`].
pub struct Receiver<IoHandler> {
    io_handler: IoHandler,
    remote_pubkey: PublicKey,
    state: ReceiveState,
    terminate: Arc<AtomicBool>,
}

impl<IoHandler> Receiver<IoHandler> {
    /// Returns the remote pubkey. Panics if there's no key.
    pub const fn remote_pubkey(&self) -> PublicKey {
        self.remote_pubkey
    }
}

impl<IoHandler: Read> Read for Receiver<IoHandler> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        checked_io!(
            self.terminate,
            read_and_decrypt(&mut self.io_handler, &mut self.state, buf)
        )
    }
}

/// Returns `remote_eph_pubkey`
fn share_eph_pubkey<IoHandler: Read + Write + Send + Sync>(
    handler: &mut IoHandler,
    local_eph_pubkey: &EphemeralPublic,
    protocol_version: Version,
) -> Result<EphemeralPublic, Error> {
    // Send our pubkey and receive theirs in tandem.
    // TODO(ismail): on the go side this is done in parallel, here we do send and receive after
    // each other. thread::spawn would require a static lifetime.
    // Should still work though.
    handler.write_all(&protocol_version.encode_initial_handshake(local_eph_pubkey))?;

    let mut response_len = 0_u8;
    handler.read_exact(slice::from_mut(&mut response_len))?;

    let mut buf = vec![0; response_len as usize];
    handler.read_exact(&mut buf)?;
    protocol_version.decode_initial_handshake(&buf)
}

/// Sign the challenge with the local private key
fn sign_challenge(
    challenge: &[u8; 32],
    local_privkey: &dyn Signer<ed25519::Signature>,
) -> Result<ed25519::Signature, Error> {
    local_privkey.try_sign(challenge).map_err(Error::signature)
}

// TODO(ismail): change from DecodeError to something more generic
// this can also fail while writing / sending
fn share_auth_signature<IoHandler: Read + Write + Send + Sync>(
    sc: &mut SecretConnection<IoHandler>,
    pubkey: &ed25519::PublicKey,
    local_signature: &ed25519::Signature,
) -> Result<proto::p2p::AuthSigMessage, Error> {
    let buf = sc
        .protocol_version
        .encode_auth_signature(pubkey, local_signature);

    sc.write_all(&buf)?;

    let mut buf = vec![0; sc.protocol_version.auth_sig_msg_response_len()];
    sc.read_exact(&mut buf)?;
    sc.protocol_version.decode_auth_signature(&buf)
}

/// Return is of the form lo, hi
#[must_use]
pub fn sort32(first: [u8; 32], second: [u8; 32]) -> ([u8; 32], [u8; 32]) {
    if second > first {
        (first, second)
    } else {
        (second, first)
    }
}

/// Encrypt AEAD authenticated data
#[allow(clippy::cast_possible_truncation)]
fn encrypt(
    chunk: &[u8],
    send_cipher: &ChaCha20Poly1305,
    send_nonce: &Nonce,
    sealed_frame: &mut [u8; TAG_SIZE + TOTAL_FRAME_SIZE],
) -> Result<(), Error> {
    assert!(!chunk.is_empty(), "chunk is empty");
    assert!(
        chunk.len() <= TOTAL_FRAME_SIZE - DATA_LEN_SIZE,
        "chunk is too big: {}! max: {}",
        chunk.len(),
        DATA_MAX_SIZE,
    );
    sealed_frame[..DATA_LEN_SIZE].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
    sealed_frame[DATA_LEN_SIZE..DATA_LEN_SIZE + chunk.len()].copy_from_slice(chunk);

    let tag = send_cipher
        .encrypt_in_place_detached(
            GenericArray::from_slice(send_nonce.to_bytes()),
            b"",
            &mut sealed_frame[..TOTAL_FRAME_SIZE],
        )
        .map_err(Error::aead)?;

    sealed_frame[TOTAL_FRAME_SIZE..].copy_from_slice(tag.as_slice());

    Ok(())
}

// Writes encrypted frames of `TAG_SIZE` + `TOTAL_FRAME_SIZE`
fn encrypt_and_write<IoHandler: Write>(
    io_handler: &mut IoHandler,
    send_state: &mut SendState,
    data: &[u8],
) -> io::Result<usize> {
    let mut n = 0_usize;
    let mut data_copy = data;
    while !data_copy.is_empty() {
        let chunk: &[u8];
        if DATA_MAX_SIZE < data.len() {
            chunk = &data[..DATA_MAX_SIZE];
            data_copy = &data_copy[DATA_MAX_SIZE..];
        } else {
            chunk = data_copy;
            data_copy = &[0_u8; 0];
        }
        let sealed_frame = &mut [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE];
        encrypt(chunk, &send_state.cipher, &send_state.nonce, sealed_frame)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        send_state.nonce.increment();
        // end encryption

        io_handler.write_all(&sealed_frame[..])?;
        n = n
            .checked_add(chunk.len())
            .expect("overflow when adding chunk lengths");
    }

    Ok(n)
}

/// Decrypt AEAD authenticated data
fn decrypt(
    ciphertext: &[u8],
    recv_cipher: &ChaCha20Poly1305,
    recv_nonce: &Nonce,
    out: &mut [u8],
) -> Result<usize, Error> {
    if ciphertext.len() < TAG_SIZE {
        return Err(Error::short_ciphertext(TAG_SIZE));
    }

    // Split ChaCha20 ciphertext from the Poly1305 tag
    let (ct, tag) = ciphertext.split_at(ciphertext.len() - TAG_SIZE);

    if out.len() < ct.len() {
        return Err(Error::small_output_buffer());
    }

    let in_out = &mut out[..ct.len()];
    in_out.copy_from_slice(ct);

    recv_cipher
        .decrypt_in_place_detached(
            GenericArray::from_slice(recv_nonce.to_bytes()),
            b"",
            in_out,
            tag.into(),
        )
        .map_err(Error::aead)?;

    Ok(in_out.len())
}

fn read_and_decrypt<IoHandler: Read>(
    io_handler: &mut IoHandler,
    recv_state: &mut ReceiveState,
    data: &mut [u8],
) -> io::Result<usize> {
    if !recv_state.buffer.is_empty() {
        let n = cmp::min(data.len(), recv_state.buffer.len());
        data.copy_from_slice(&recv_state.buffer[..n]);
        let mut leftover_portion = vec![
            0;
            recv_state
                .buffer
                .len()
                .checked_sub(n)
                .expect("leftover calculation failed")
        ];
        leftover_portion.clone_from_slice(&recv_state.buffer[n..]);
        recv_state.buffer = leftover_portion;

        return Ok(n);
    }

    let mut sealed_frame = [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE];
    io_handler.read_exact(&mut sealed_frame)?;

    // decrypt the frame
    let mut frame = [0_u8; TOTAL_FRAME_SIZE];
    let res = decrypt(
        &sealed_frame,
        &recv_state.cipher,
        &recv_state.nonce,
        &mut frame,
    );

    if let Err(err) = res {
        return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
    }

    recv_state.nonce.increment();
    // end decryption

    let chunk_length = u32::from_le_bytes(frame[..4].try_into().expect("chunk framing failed"));

    if chunk_length as usize > DATA_MAX_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("chunk is too big: {}! max: {}", chunk_length, DATA_MAX_SIZE),
        ));
    }

    let mut chunk = vec![0; chunk_length as usize];
    chunk.clone_from_slice(
        &frame[DATA_LEN_SIZE
            ..(DATA_LEN_SIZE
                .checked_add(chunk_length as usize)
                .expect("chunk size addition overflow"))],
    );

    let n = cmp::min(data.len(), chunk.len());
    data[..n].copy_from_slice(&chunk[..n]);
    recv_state.buffer.copy_from_slice(&chunk[n..]);

    Ok(n)
}
