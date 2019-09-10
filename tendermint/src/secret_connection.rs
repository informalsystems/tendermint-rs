//! `SecretConnection`: Transport layer encryption for Tendermint P2P connections.

mod kdf;
mod nonce;
mod public_key;

pub use self::{kdf::Kdf, nonce::Nonce, public_key::PublicKey};
use crate::{
    amino_types::AuthSigMessage,
    error::{Error, ErrorKind},
};
use byteorder::{ByteOrder, LE};
use bytes::BufMut;
use prost::{encoding::encode_varint, Message};
use rand_os::OsRng;
use ring::aead;
use signatory::{ed25519, Signature, Signer, Verifier};
use signatory_dalek::Ed25519Verifier;
use std::{
    cmp,
    io::{self, Read, Write},
    marker::{Send, Sync},
};
use subtle::ConstantTimeEq;
use x25519_dalek::{EphemeralSecret, PublicKey as EphemeralPublic};

/// Size of the MAC tag
pub const TAG_SIZE: usize = 16;

/// 4 + 1024 == 1028 total frame size
const DATA_LEN_SIZE: usize = 4;
const DATA_MAX_SIZE: usize = 1024;
const TOTAL_FRAME_SIZE: usize = DATA_MAX_SIZE + DATA_LEN_SIZE;

/// Encrypted connection between peers in a Tendermint network
pub struct SecretConnection<IoHandler: Read + Write + Send + Sync> {
    io_handler: IoHandler,
    recv_nonce: Nonce,
    send_nonce: Nonce,
    recv_secret: aead::OpeningKey,
    send_secret: aead::SealingKey,
    remote_pubkey: PublicKey,
    recv_buffer: Vec<u8>,
}

impl<IoHandler: Read + Write + Send + Sync> SecretConnection<IoHandler> {
    /// Returns authenticated remote pubkey
    pub fn remote_pubkey(&self) -> PublicKey {
        self.remote_pubkey
    }
    #[allow(clippy::new_ret_no_self)]
    /// Performs handshake and returns a new authenticated SecretConnection.
    pub fn new(
        mut handler: IoHandler,
        local_pubkey: &PublicKey,
        local_privkey: &dyn Signer<ed25519::Signature>,
    ) -> Result<SecretConnection<IoHandler>, Error> {
        // Generate ephemeral keys for perfect forward secrecy.
        let (local_eph_pubkey, local_eph_privkey) = gen_eph_keys();

        // Write local ephemeral pubkey and receive one too.
        // NOTE: every 32-byte string is accepted as a Curve25519 public key
        // (see DJB's Curve25519 paper: http://cr.yp.to/ecdh/curve25519-20060209.pdf)
        let remote_eph_pubkey = share_eph_pubkey(&mut handler, &local_eph_pubkey)?;

        // Compute common shared secret.
        let shared_secret = EphemeralSecret::diffie_hellman(local_eph_privkey, &remote_eph_pubkey);

        // Reject all-zero outputs from X25519 (i.e. from low-order points)
        //
        // See the following for information on potential attacks this check
        // aids in mitigating:
        //
        // - https://github.com/tendermint/kms/issues/142
        // - https://eprint.iacr.org/2019/526.pdf
        if shared_secret.as_bytes().ct_eq(&[0x00; 32]).unwrap_u8() == 1 {
            Err(ErrorKind::InvalidKey)?;
        }

        // Sort by lexical order.
        let local_eph_pubkey_bytes = *local_eph_pubkey.as_bytes();
        let (low_eph_pubkey_bytes, _) =
            sort32(local_eph_pubkey_bytes, *remote_eph_pubkey.as_bytes());

        // Check if the local ephemeral public key
        // was the least, lexicographically sorted.
        let loc_is_least = local_eph_pubkey_bytes == low_eph_pubkey_bytes;

        let kdf = Kdf::derive_secrets_and_challenge(shared_secret.as_bytes(), loc_is_least);

        // Construct SecretConnection.
        let mut sc = SecretConnection {
            io_handler: handler,
            recv_buffer: vec![],
            recv_nonce: Nonce::default(),
            send_nonce: Nonce::default(),
            recv_secret: aead::OpeningKey::new(&aead::CHACHA20_POLY1305, &kdf.recv_secret)
                .map_err(|_| ErrorKind::Crypto)?,
            send_secret: aead::SealingKey::new(&aead::CHACHA20_POLY1305, &kdf.send_secret)
                .map_err(|_| ErrorKind::Crypto)?,
            remote_pubkey: PublicKey::from(
                ed25519::PublicKey::from_bytes(remote_eph_pubkey.as_bytes())
                    .ok_or_else(|| ErrorKind::Crypto)?,
            ),
        };

        // Sign the challenge bytes for authentication.
        let local_signature = sign_challenge(&kdf.challenge, local_privkey)?;

        // Share (in secret) each other's pubkey & challenge signature
        let auth_sig_msg = match local_pubkey {
            PublicKey::Ed25519(ref pk) => {
                share_auth_signature(&mut sc, pk.as_bytes(), local_signature)?
            }
        };

        let remote_pubkey =
            ed25519::PublicKey::from_bytes(&auth_sig_msg.key).ok_or_else(|| ErrorKind::Crypto)?;
        let remote_signature: &[u8] = &auth_sig_msg.sig;
        let remote_sig = ed25519::Signature::from_bytes(remote_signature)?;

        let remote_verifier = Ed25519Verifier::from(&remote_pubkey);
        remote_verifier.verify(&kdf.challenge, &remote_sig)?;

        // We've authorized.
        sc.remote_pubkey = PublicKey::from(remote_pubkey);

        Ok(sc)
    }

    /// Unseal (i.e. decrypt) AEAD authenticated data
    fn open(&self, ciphertext: &[u8], out: &mut [u8]) -> Result<usize, Error> {
        let nonce = aead::Nonce::from(&self.recv_nonce);
        let associated_data = aead::Aad::empty();

        // optimize if the provided buffer is sufficiently large
        let len = if out.len() >= ciphertext.len() {
            let in_out = &mut out[..ciphertext.len()];
            in_out.copy_from_slice(ciphertext);

            aead::open_in_place(&self.recv_secret, nonce, associated_data, 0, in_out)
                .map_err(|_| ErrorKind::Crypto)?
                .len()
        } else {
            let mut in_out = ciphertext.to_vec();
            let out0 =
                aead::open_in_place(&self.recv_secret, nonce, aead::Aad::empty(), 0, &mut in_out)
                    .map_err(|_| ErrorKind::Crypto)?;
            out[..out0.len()].copy_from_slice(out0);
            out0.len()
        };

        Ok(len)
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
            aead::Nonce::from(&self.send_nonce),
            aead::Aad::empty(),
            sealed_frame,
            TAG_SIZE,
        )
        .map_err(|_| ErrorKind::Crypto)?;

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
        let res = self.open(&sealed_frame, &mut frame);

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

        if chunk_length as usize > DATA_MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "chunk_length is greater than dataMaxSize",
            ));
        }

        let mut chunk = vec![0; chunk_length as usize];
        chunk.clone_from_slice(
            &frame[DATA_LEN_SIZE..(DATA_LEN_SIZE.checked_add(chunk_length as usize).unwrap())],
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

/// Returns pubkey, private key
fn gen_eph_keys() -> (EphemeralPublic, EphemeralSecret) {
    let mut local_csprng = OsRng::new().unwrap();
    let local_privkey = EphemeralSecret::new(&mut local_csprng);
    let local_pubkey = EphemeralPublic::from(&local_privkey);
    (local_pubkey, local_privkey)
}

/// Returns remote_eph_pubkey
fn share_eph_pubkey<IoHandler: Read + Write + Send + Sync>(
    handler: &mut IoHandler,
    local_eph_pubkey: &EphemeralPublic,
) -> Result<EphemeralPublic, Error> {
    // Send our pubkey and receive theirs in tandem.
    // TODO(ismail): on the go side this is done in parallel, here we do send and receive after
    // each other. thread::spawn would require a static lifetime.
    // Should still work though.

    let mut buf = vec![0; 0];
    let local_eph_pubkey_vec = local_eph_pubkey.as_bytes();
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
        Err(ErrorKind::Protocol)?;
    }
    // after total length (33) and byte length (32), we expect the raw bytes
    // of the pub key:
    remote_eph_pubkey_fixed.copy_from_slice(&buf[2..34]);

    if is_blacklisted_point(&remote_eph_pubkey_fixed) {
        Err(ErrorKind::InvalidKey.into())
    } else {
        Ok(EphemeralPublic::from(remote_eph_pubkey_fixed))
    }
}

/// Reject the blacklist of degenerate points listed on <https://cr.yp.to/ecdh.html>
///
/// These points contain low-order elements. Rejecting them is suggested in
/// the "May the Fourth" paper under Section 5: Software Countermeasures
/// (see "Rejecting Known Bad Points" subsection):
///
/// <https://eprint.iacr.org/2017/806.pdf>
fn is_blacklisted_point(point: &[u8; 32]) -> bool {
    // Note: as these are public points and do not interact with secret-key
    // material in any way, this check does not need to be performed in
    // constant-time.
    match point {
        // 0 (order 4)
        &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            true
        }

        // 1 (order 1)
        [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] => {
            true
        }

        // 325606250916557431795983626356110631294008115727848805560023387167927233504 (order 8)
        &[0xe0, 0xeb, 0x7a, 0x7c, 0x3b, 0x41, 0xb8, 0xae, 0x16, 0x56, 0xe3, 0xfa, 0xf1, 0x9f, 0xc4, 0x6a, 0xda, 0x09, 0x8d, 0xeb, 0x9c, 0x32, 0xb1, 0xfd, 0x86, 0x62, 0x05, 0x16, 0x5f, 0x49, 0xb8, 0x00] => {
            true
        }

        // 39382357235489614581723060781553021112529911719440698176882885853963445705823 (order 8)
        &[0x5f, 0x9c, 0x95, 0xbc, 0xa3, 0x50, 0x8c, 0x24, 0xb1, 0xd0, 0xb1, 0x55, 0x9c, 0x83, 0xef, 0x5b, 0x04, 0x44, 0x5c, 0xc4, 0x58, 0x1c, 0x8e, 0x86, 0xd8, 0x22, 0x4e, 0xdd, 0xd0, 0x9f, 0x11, 0x57] => {
            true
        }

        // p - 1 (order 2)
        [0xec, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f] => {
            true
        }

        // p (order 4) */
        [0xed, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f] => {
            true
        }

        // p + 1 (order 1)
        [0xee, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f] => {
            true
        }
        _ => false,
    }
}

/// Return is of the form lo, hi
fn sort32(first: [u8; 32], second: [u8; 32]) -> ([u8; 32], [u8; 32]) {
    if second > first {
        (first, second)
    } else {
        (second, first)
    }
}

/// Sign the challenge with the local private key
fn sign_challenge(
    challenge: &[u8; 32],
    local_privkey: &dyn Signer<ed25519::Signature>,
) -> Result<ed25519::Signature, Error> {
    local_privkey
        .try_sign(challenge)
        .map_err(|_| ErrorKind::Crypto.into())
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
