//! `SecretConnection`: Transport layer encryption for Tendermint P2P connections.

use std::{
    cmp,
    convert::{TryFrom, TryInto},
    io::{self, Read, Write},
    marker::{Send, Sync},
    slice,
};

use crate::error::{self, Error};
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
            None => return Err(error::missing_secret_error()),
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
            return Err(error::low_order_key_error());
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
            .ok_or_else(error::missing_key_error)?;

        let remote_pubkey = match pk_sum {
            proto::crypto::public_key::Sum::Ed25519(ref bytes) => {
                ed25519::PublicKey::from_bytes(bytes).map_err(error::signature_error)
            }
            proto::crypto::public_key::Sum::Secp256k1(_) => Err(error::unsupported_key_error()),
        }?;

        let remote_sig = ed25519::Signature::try_from(auth_sig_msg.sig.as_slice())
            .map_err(error::signature_error)?;

        if self.protocol_version.has_transcript() {
            remote_pubkey
                .verify(&self.state.sc_mac, &remote_sig)
                .map_err(error::signature_error)?;
        } else {
            remote_pubkey
                .verify(&self.state.kdf.challenge, &remote_sig)
                .map_err(error::signature_error)?;
        }

        // We've authorized.
        Ok(remote_pubkey.into())
    }
}

/// Encrypted connection between peers in a Tendermint network.
pub struct SecretConnection<IoHandler: Read + Write + Send + Sync> {
    io_handler: IoHandler,
    protocol_version: Version,
    recv_nonce: Nonce,
    send_nonce: Nonce,
    recv_cipher: ChaCha20Poly1305,
    send_cipher: ChaCha20Poly1305,
    remote_pubkey: Option<PublicKey>,
    recv_buffer: Vec<u8>,
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
            recv_buffer: vec![],
            recv_nonce: Nonce::default(),
            send_nonce: Nonce::default(),
            recv_cipher: h.state.recv_cipher.clone(),
            send_cipher: h.state.send_cipher.clone(),
            remote_pubkey: None,
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

    /// Encrypt AEAD authenticated data
    #[allow(clippy::cast_possible_truncation)]
    fn encrypt(
        &self,
        chunk: &[u8],
        sealed_frame: &mut [u8; TAG_SIZE + TOTAL_FRAME_SIZE],
    ) -> Result<(), Error> {
        debug_assert!(!chunk.is_empty(), "chunk is empty");
        debug_assert!(
            chunk.len() <= TOTAL_FRAME_SIZE - DATA_LEN_SIZE,
            "chunk is too big: {}! max: {}",
            chunk.len(),
            DATA_MAX_SIZE,
        );
        sealed_frame[..DATA_LEN_SIZE].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
        sealed_frame[DATA_LEN_SIZE..DATA_LEN_SIZE + chunk.len()].copy_from_slice(chunk);

        let tag = self
            .send_cipher
            .encrypt_in_place_detached(
                GenericArray::from_slice(self.send_nonce.to_bytes()),
                b"",
                &mut sealed_frame[..TOTAL_FRAME_SIZE],
            )
            .map_err(error::aead_error)?;

        sealed_frame[TOTAL_FRAME_SIZE..].copy_from_slice(tag.as_slice());

        Ok(())
    }

    /// Decrypt AEAD authenticated data
    fn decrypt(&self, ciphertext: &[u8], out: &mut [u8]) -> Result<usize, Error> {
        if ciphertext.len() < TAG_SIZE {
            return Err(error::short_ciphertext_error(TAG_SIZE));
        }

        // Split ChaCha20 ciphertext from the Poly1305 tag
        let (ct, tag) = ciphertext.split_at(ciphertext.len() - TAG_SIZE);

        if out.len() < ct.len() {
            return Err(error::small_output_buffer_error());
        }

        let in_out = &mut out[..ct.len()];
        in_out.copy_from_slice(ct);

        self.recv_cipher
            .decrypt_in_place_detached(
                GenericArray::from_slice(self.recv_nonce.to_bytes()),
                b"",
                in_out,
                tag.into(),
            )
            .map_err(error::aead_error)?;

        Ok(in_out.len())
    }
}

impl<IoHandler> Read for SecretConnection<IoHandler>
where
    IoHandler: Read + Write + Send + Sync,
{
    // CONTRACT: data smaller than DATA_MAX_SIZE is read atomically.
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
        if !self.recv_buffer.is_empty() {
            let n = cmp::min(data.len(), self.recv_buffer.len());
            data.copy_from_slice(&self.recv_buffer[..n]);
            let mut leftover_portion = vec![
                0;
                self.recv_buffer
                    .len()
                    .checked_sub(n)
                    .expect("leftover calculation failed")
            ];
            leftover_portion.clone_from_slice(&self.recv_buffer[n..]);
            self.recv_buffer = leftover_portion;

            return Ok(n);
        }

        let mut sealed_frame = [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE];
        self.io_handler.read_exact(&mut sealed_frame)?;

        // decrypt the frame
        let mut frame = [0_u8; TOTAL_FRAME_SIZE];
        let res = self.decrypt(&sealed_frame, &mut frame);

        if let Err(err) = res {
            return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
        }

        self.recv_nonce.increment();
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
        self.recv_buffer.copy_from_slice(&chunk[n..]);

        Ok(n)
    }
}

impl<IoHandler> Write for SecretConnection<IoHandler>
where
    IoHandler: Read + Write + Send + Sync,
{
    // Writes encrypted frames of `TAG_SIZE` + `TOTAL_FRAME_SIZE`
    // CONTRACT: data smaller than DATA_MAX_SIZE is read atomically.
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
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
            let res = self.encrypt(chunk, sealed_frame);
            if let Err(err) = res {
                return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
            }
            self.send_nonce.increment();
            // end encryption

            self.io_handler.write_all(&sealed_frame[..])?;
            n = n
                .checked_add(chunk.len())
                .expect("overflow when adding chunk lenghts");
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.io_handler.flush()
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
    handler
        .write_all(&protocol_version.encode_initial_handshake(local_eph_pubkey))
        .map_err(error::io_error)?;

    let mut response_len = 0_u8;
    handler
        .read_exact(slice::from_mut(&mut response_len))
        .map_err(error::io_error)?;

    let mut buf = vec![0; response_len as usize];
    handler.read_exact(&mut buf).map_err(error::io_error)?;
    protocol_version.decode_initial_handshake(&buf)
}

/// Sign the challenge with the local private key
fn sign_challenge(
    challenge: &[u8; 32],
    local_privkey: &dyn Signer<ed25519::Signature>,
) -> Result<ed25519::Signature, Error> {
    local_privkey
        .try_sign(challenge)
        .map_err(error::signature_error)
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

    sc.write_all(&buf).map_err(error::io_error)?;

    let mut buf = vec![0; sc.protocol_version.auth_sig_msg_response_len()];
    sc.read_exact(&mut buf).map_err(error::io_error)?;

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
