//! `SecretConnection`: Transport layer encryption for Tendermint P2P connections.

extern crate hkdf;
extern crate rand;
extern crate ring;
extern crate x25519_dalek;

mod kdf;
mod nonce;

use self::{
    rand::OsRng,
    ring::aead,
    x25519_dalek::{EphemeralPublic, EphemeralSecret},
};
use crate::{
    amino_types::AuthSigMessage,
    byteorder::{ByteOrder, LE},
    bytes::BufMut,
    error::Error,
    prost::{encoding::encode_varint, Message},
    public_keys::SecretConnectionKey,
    signatory::{ed25519, Signature, Signer},
    signatory_dalek::Ed25519Verifier,
};
use std::{
    cmp,
    io::{self, Read, Write},
    marker::{Send, Sync},
};

pub use self::{kdf::Kdf, nonce::Nonce};

/// 4 + 1024 == 1028 total frame size
const DATA_LEN_SIZE: usize = 4;
const DATA_MAX_SIZE: usize = 1024;
const TOTAL_FRAME_SIZE: usize = DATA_MAX_SIZE + DATA_LEN_SIZE;

/// Size of the MAC tag
pub const TAG_SIZE: usize = 16;

/// Encrypted connection between peers in a Tendermint network
pub struct SecretConnection<IoHandler: Read + Write + Send + Sync> {
    io_handler: IoHandler,
    recv_nonce: Nonce,
    send_nonce: Nonce,
    recv_secret: aead::OpeningKey,
    send_secret: aead::SealingKey,
    remote_pubkey: SecretConnectionKey,
    recv_buffer: Vec<u8>,
}

impl<IoHandler: Read + Write + Send + Sync> SecretConnection<IoHandler> {
    /// Returns authenticated remote pubkey
    pub fn remote_pubkey(&self) -> SecretConnectionKey {
        self.remote_pubkey
    }
    #[allow(clippy::new_ret_no_self)]
    /// Performs handshake and returns a new authenticated SecretConnection.
    pub fn new(
        mut handler: IoHandler,
        local_pubkey: &SecretConnectionKey,
        local_privkey: &dyn Signer<ed25519::Signature>,
    ) -> Result<SecretConnection<IoHandler>, Error> {
        // Generate ephemeral keys for perfect forward secrecy.
        let (local_eph_pubkey, local_eph_privkey) = gen_eph_keys();

        // Write local ephemeral pubkey and receive one too.
        // NOTE: every 32-byte string is accepted as a Curve25519 public key
        // (see DJB's Curve25519 paper: http://cr.yp.to/ecdh/curve25519-20060209.pdf)
        let remote_eph_pubkey = share_eph_pubkey(&mut handler, &local_eph_pubkey)?;

        // Compute common shared secret.
        //let shared_secret = diffie_hellman(&local_eph_privkey, &remote_eph_pubkey);
        let shared_secret = EphemeralSecret::diffie_hellman(local_eph_privkey, &remote_eph_pubkey);

        // Sort by lexical order.
        let (low_eph_pubkey, _) = sort32(local_eph_pubkey, remote_eph_pubkey);

        // Check if the local ephemeral public key
        // was the least, lexicographically sorted.
        let loc_is_least = local_eph_pubkey == low_eph_pubkey;

        let kdf = Kdf::derive_secrets_and_challenge(&shared_secret, loc_is_least);

        // Construct SecretConnection.
        let mut sc = SecretConnection {
            io_handler: handler,
            recv_buffer: vec![],
            recv_nonce: Nonce::default(),
            send_nonce: Nonce::default(),
            recv_secret: aead::OpeningKey::new(&aead::CHACHA20_POLY1305, &kdf.recv_secret)
                .map_err(|_| Error::Crypto)?,
            send_secret: aead::SealingKey::new(&aead::CHACHA20_POLY1305, &kdf.send_secret)
                .map_err(|_| Error::Crypto)?,
            remote_pubkey: SecretConnectionKey::from(ed25519::PublicKey::from_bytes(
                &remote_eph_pubkey,
            )?),
        };

        // Sign the challenge bytes for authentication.
        let local_signature = sign_challenge(kdf.challenge, local_privkey)?;

        // Share (in secret) each other's pubkey & challenge signature
        let auth_sig_msg = match local_pubkey {
            SecretConnectionKey::Ed25519(ref pk) => {
                share_auth_signature(&mut sc, pk.as_bytes(), local_signature)?
            }
        };

        let remote_pubkey = ed25519::PublicKey::from_bytes(&auth_sig_msg.key)?;
        let remote_signature: &[u8] = &auth_sig_msg.sig;
        let remote_sig = ed25519::Signature::from_bytes(remote_signature)?;

        let remote_verifier = Ed25519Verifier::from(&remote_pubkey);
        ed25519::verify(&remote_verifier, &kdf.challenge, &remote_sig)?;

        // We've authorized.
        sc.remote_pubkey = SecretConnectionKey::from(remote_pubkey);

        Ok(sc)
    }

    /// Unseal (i.e. decrypt) AEAD authenticated data
    fn open(&self, authtext: &[u8], ciphertext: &[u8], out: &mut [u8]) -> Result<usize, Error> {
        // optimize if the provided buffer is sufficiently large
        if out.len() >= ciphertext.len() {
            let in_out = &mut out[..ciphertext.len()];
            in_out.copy_from_slice(ciphertext);
            let len = aead::open_in_place(
                &self.recv_secret,
                &self.recv_nonce.to_bytes(),
                authtext,
                0,
                in_out,
            )
            .map_err(|_| Error::Crypto)?
            .len();
            Ok(len)
        } else {
            let mut in_out = ciphertext.to_vec();
            let out0 = aead::open_in_place(
                &self.recv_secret,
                &self.recv_nonce.to_bytes(),
                authtext,
                0,
                &mut in_out,
            )
            .map_err(|_| Error::Crypto)?;
            out[..out0.len()].copy_from_slice(out0);
            Ok(out0.len())
        }
    }

    /// Seal (i.e. encrypt) AEAD authenticated data
    fn seal(
        &self,
        chunk: &[u8],
        sealed_frame: &mut [u8; TAG_SIZE + TOTAL_FRAME_SIZE],
    ) -> Result<(), Error> {
        let chunk_length = chunk.len();
        let mut frame = [0u8; TOTAL_FRAME_SIZE];
        LE::write_u32(&mut frame[..DATA_LEN_SIZE], chunk_length as u32);
        frame[DATA_LEN_SIZE..DATA_LEN_SIZE + chunk_length].copy_from_slice(chunk);
        sealed_frame[..frame.len()].copy_from_slice(&frame);

        aead::seal_in_place(
            &self.send_secret,
            &self.send_nonce.to_bytes(),
            &[0u8; 0],
            sealed_frame,
            TAG_SIZE,
        )
        .map_err(|_| Error::Crypto)?;

        Ok(())
    }
}

impl<IoHandler> Read for SecretConnection<IoHandler>
where
    IoHandler: Read + Write + Send + Sync,
{
    // CONTRACT: data smaller than dataMaxSize is read atomically.
    fn read(&mut self, data: &mut [u8]) -> Result<usize, io::Error> {
        if !self.recv_buffer.is_empty() {
            let n = cmp::min(data.len(), self.recv_buffer.len());
            data.copy_from_slice(&self.recv_buffer[..n]);
            let mut leftover_portion = vec![0; self.recv_buffer.len().checked_sub(n).unwrap()];
            leftover_portion.clone_from_slice(&self.recv_buffer[n..]);
            self.recv_buffer = leftover_portion;

            return Ok(n);
        }

        let mut sealed_frame = [0u8; TAG_SIZE + TOTAL_FRAME_SIZE];
        self.io_handler.read_exact(&mut sealed_frame)?;

        // decrypt the frame
        let mut frame = [0u8; TOTAL_FRAME_SIZE];
        let res = self.open(&[0u8; 0], &sealed_frame, &mut frame);
        let mut frame_copy = [0u8; TOTAL_FRAME_SIZE];
        frame_copy.clone_from_slice(&frame);
        if res.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                res.err().unwrap().to_string(),
            ));
        }
        self.recv_nonce.increment();
        // end decryption

        let mut chunk_length_specifier = vec![0; 4];
        chunk_length_specifier.clone_from_slice(&frame[..4]);

        let chunk_length = LE::read_u32(&chunk_length_specifier);
        if chunk_length > DATA_MAX_SIZE as u32 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "chunk_length is greater than dataMaxSize",
            ))
        } else {
            let mut chunk = vec![0; chunk_length as usize];
            chunk.clone_from_slice(
                &frame_copy
                    [DATA_LEN_SIZE..(DATA_LEN_SIZE.checked_add(chunk_length as usize).unwrap())],
            );
            let n = cmp::min(data.len(), chunk.len());
            data[..n].copy_from_slice(&chunk[..n]);
            self.recv_buffer.copy_from_slice(&chunk[n..]);

            Ok(n)
        }
    }
}

impl<IoHandler> Write for SecretConnection<IoHandler>
where
    IoHandler: Read + Write + Send + Sync,
{
    // Writes encrypted frames of `sealedFrameSize`
    // CONTRACT: data smaller than dataMaxSize is read atomically.
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        let mut n = 0usize;
        let mut data_copy = &data[..];
        while !data_copy.is_empty() {
            let chunk: &[u8];
            if DATA_MAX_SIZE < data.len() {
                chunk = &data[..DATA_MAX_SIZE];
                data_copy = &data_copy[DATA_MAX_SIZE..];
            } else {
                chunk = data_copy;
                data_copy = &[0u8; 0];
            }
            let sealed_frame = &mut [0u8; TAG_SIZE + TOTAL_FRAME_SIZE];
            let res = self.seal(chunk, sealed_frame);
            if res.is_err() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    res.err().unwrap().to_string(),
                ));
            }
            self.send_nonce.increment();
            // end encryption

            self.io_handler.write_all(&sealed_frame[..])?;
            n = n.checked_add(chunk.len()).unwrap();
        }

        Ok(n)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.io_handler.flush()
    }
}

// Returns pubkey, private key
fn gen_eph_keys() -> (EphemeralPublic, EphemeralSecret) {
    let mut local_csprng = OsRng::new().unwrap();
    let local_privkey = EphemeralSecret::new(&mut local_csprng);
    let local_pubkey = EphemeralPublic::from(&local_privkey);
    (local_pubkey, local_privkey)
}

// Returns remote_eph_pubkey
fn share_eph_pubkey<IoHandler: Read + Write + Send + Sync>(
    handler: &mut IoHandler,
    local_eph_pubkey: &EphemeralPublic,
) -> Result<EphemeralPublic, Error> {
    // Send our pubkey and receive theirs in tandem.
    // TODO(ismail): on the go side this is done in parallel, here we do send and receive after
    // each other. thread::spawn would require a static lifetime.
    // Should still work though.

    let mut buf = vec![0; 0];
    let local_eph_pubkey_vec = &local_eph_pubkey.0.to_bytes();
    // Note: this is not regular protobuf encoding but raw length prefixed amino encoding;
    // amino prefixes with the total length, and the raw bytes array's length, too:
    encode_varint((local_eph_pubkey_vec.len() + 1) as u64, &mut buf); // 33
    encode_varint(local_eph_pubkey_vec.len() as u64, &mut buf); // 32
    buf.put_slice(local_eph_pubkey_vec); // raw bytes

    // TODO(ismail): we probably do *not* need the double length delimiting here or in tendermint)
    // this is the sending part of:
    // https://github.com/tendermint/tendermint/blob/013b9cef642f875634c614019ab13b17570778ad/p2p/conn/secret_connection.go#L208-L238
    handler.write_all(&buf)?;

    let mut buf = vec![0; 34];
    handler.read_exact(&mut buf)?;

    // this is the receiving part of:
    // https://github.com/tendermint/tendermint/blob/013b9cef642f875634c614019ab13b17570778ad/p2p/conn/secret_connection.go#L208-L238
    let mut remote_eph_pubkey_fixed: [u8; 32] = Default::default();
    if buf[0] != 33 || buf[1] != 32 {
        return Err(Error::Protocol);
    }
    // after total length (33) and byte length (32), we expect the raw bytes
    // of the pub key:
    remote_eph_pubkey_fixed.copy_from_slice(&buf[2..34]);

    Ok(remote_eph_pubkey_fixed)
}

// Return is of the form lo, hi
fn sort32(first: [u8; 32], second: [u8; 32]) -> ([u8; 32], [u8; 32]) {
    if second > first {
        (first, second)
    } else {
        (second, first)
    }
}

// Sign the challenge with the local private key
fn sign_challenge(
    challenge: [u8; 32],
    local_privkey: &dyn Signer<ed25519::Signature>,
) -> Result<ed25519::Signature, Error> {
    ed25519::sign(local_privkey, &challenge).map_err(|_| Error::Crypto)
}

// TODO(ismail): change from DecodeError to something more generic
// this can also fail while writing / sending
fn share_auth_signature<IoHandler: Read + Write + Send + Sync>(
    sc: &mut SecretConnection<IoHandler>,
    pubkey: &[u8; 32],
    signature: ed25519::Signature,
) -> Result<AuthSigMessage, Error> {
    let amsg = AuthSigMessage {
        key: pubkey.to_vec(),
        sig: signature.into_bytes().to_vec(),
    };
    let mut buf: Vec<u8> = vec![];
    amsg.encode_length_delimited(&mut buf)?;
    sc.write_all(&buf)?;

    let mut rbuf = vec![0; 106]; // 100 = 32 + 64 + (amino overhead = 2 fields + 2 lengths + 4 prefix bytes + total length)
    sc.read_exact(&mut rbuf)?;

    // TODO: proper error handling:
    Ok(AuthSigMessage::decode_length_delimited(&rbuf)?)
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_sort() {
        // sanity check
        let t1 = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let t2 = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];
        let (ref t3, ref t4) = sort32(t1, t2);
        assert_eq!(t1, *t3);
        assert_eq!(t2, *t4);
    }

    #[test]
    fn test_dh_compatibility() {
        let local_priv = &[
            15, 54, 189, 54, 63, 255, 158, 244, 56, 168, 155, 63, 246, 79, 208, 192, 35, 194, 39,
            232, 170, 187, 179, 36, 65, 36, 237, 12, 225, 176, 201, 54,
        ];
        let remote_pub = &[
            193, 34, 183, 46, 148, 99, 179, 185, 242, 148, 38, 40, 37, 150, 76, 251, 25, 51, 46,
            143, 189, 201, 169, 218, 37, 136, 51, 144, 88, 196, 10, 20,
        ];

        // generated using computeDHSecret in go
        let expected_dh = &[
            92, 56, 205, 118, 191, 208, 49, 3, 226, 150, 30, 205, 230, 157, 163, 7, 36, 28, 223,
            84, 165, 43, 78, 38, 126, 200, 40, 217, 29, 36, 43, 37,
        ];
        let got_dh = diffie_hellman(local_priv, remote_pub);

        assert_eq!(expected_dh, &got_dh);
    }
}
