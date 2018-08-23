use byteorder::{BigEndian, ByteOrder};
use bytes::BufMut;
use error::Error;
use hkdf::Hkdf;
use prost::encoding::bytes::merge;
use prost::encoding::encode_varint;
use prost::encoding::WireType;
use prost::{DecodeError, Message};
use rand::OsRng;
use ring::aead;
use sha2::Sha256;
use signatory::ed25519::Signer;
use signatory::ed25519::{DefaultVerifier, PublicKey, Signature, Verifier};
use signatory::providers::dalek::Ed25519Signer as DalekSigner;
use std::io::{Read, Write};
use std::marker::{Send, Sync};
use std::{cmp, io, io::Cursor};
use x25519_dalek::{diffie_hellman, generate_public, generate_secret};

// 4 + 1024 == 1028 total frame size
const DATA_LEN_SIZE: usize = 4;
const DATA_MAX_SIZE: usize = 1024;
const TOTAL_FRAME_SIZE: usize = DATA_MAX_SIZE + DATA_LEN_SIZE;
// 16 is the size of the mac tag
const TAG_SIZE: usize = 16;
const AEAD_NONCE_SIZE: usize = 12;

// Implements net.Conn
pub struct SecretConnection<IoHandler: io::Read + io::Write + Send + Sync> {
    io_handler: IoHandler,
    recv_nonce: Nonce,
    send_nonce: Nonce,
    recv_secret: aead::OpeningKey,
    send_secret: aead::SealingKey,
    remote_pubkey: [u8; 32],
    recv_buffer: Vec<u8>,
}

impl<IoHandler: io::Read + io::Write + Send + Sync> SecretConnection<IoHandler> {
    // Returns authenticated remote pubkey
    fn remote_pubkey(&self) -> [u8; 32] {
        self.remote_pubkey
    }
    // Performs handshake and returns a new authenticated SecretConnection.
    pub fn new(
        mut handler: IoHandler,
        local_privkey: &DalekSigner,
    ) -> Result<SecretConnection<IoHandler>, Error> {
        // TODO: Error check
        let local_pubkey = local_privkey.public_key().unwrap();

        // Generate ephemeral keys for perfect forward secrecy.
        let (local_eph_pubkey, local_eph_privkey) = gen_eph_keys();

        // Write local ephemeral pubkey and receive one too.
        // NOTE: every 32-byte string is accepted as a Curve25519 public key
        // (see DJB's Curve25519 paper: http://cr.yp.to/ecdh/curve25519-20060209.pdf)
        let remote_eph_pubkey = share_eph_pubkey(&mut handler, &local_eph_pubkey).unwrap();

        // Compute common shared secret.
        let shared_secret = diffie_hellman(&local_eph_privkey, &remote_eph_pubkey);

        // Sort by lexical order.
        let (low_eph_pubkey, _) = sort32(local_eph_pubkey, remote_eph_pubkey);

        // Check if the local ephemeral public key
        // was the least, lexicographically sorted.
        let loc_is_least = local_eph_pubkey == low_eph_pubkey;

        let (recv_secret, send_secret, challenge) =
            derive_secrets_and_challenge(&shared_secret, loc_is_least);

        // Construct SecretConnection.
        let mut sc = SecretConnection {
            io_handler: handler,
            recv_buffer: vec![],
            recv_nonce: Nonce::default(),
            send_nonce: Nonce::default(),
            recv_secret: aead::OpeningKey::new(&aead::CHACHA20_POLY1305, &recv_secret).unwrap(),
            send_secret: aead::SealingKey::new(&aead::CHACHA20_POLY1305, &send_secret).unwrap(),
            remote_pubkey: remote_eph_pubkey,
        };

        // Sign the challenge bytes for authentication.
        // TODO: Error check
        let local_signature = sign_challenge(challenge, local_privkey).unwrap();

        // Share (in secret) each other's pubkey & challenge signature
        // TODO: Error check
        let auth_sig_msg =
            share_auth_signature(&mut sc, local_pubkey.as_bytes(), local_signature).unwrap();

        let remote_pubkey = PublicKey::from_bytes(&auth_sig_msg.key).unwrap();
        let remote_signature: &[u8] = &auth_sig_msg.sig;
        let remote_sig = Signature::from_bytes(remote_signature).unwrap();

        let valid_sig = DefaultVerifier::verify(&remote_pubkey, &challenge, &remote_sig);

        valid_sig.map_err(|e| err!(ChallengeVerification, "{}", e))?;

        // We've authorized.
        sc.remote_pubkey.copy_from_slice(&auth_sig_msg.key);

        Ok(sc)
    }

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
            ).map_err(|e| err!(AuthCryptoError, "open_in_place failed: {}", e))?
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
            ).map_err(|e| err!(AuthCryptoError, "open_in_place: {}", e))?;
            out[..out0.len()].copy_from_slice(out0);
            Ok(out0.len())
        }
    }

    fn seal(
        &self,
        chunk: &[u8],
        sealed_frame: &mut [u8; TAG_SIZE + TOTAL_FRAME_SIZE],
    ) -> Result<(), Error> {
        let chunk_length = chunk.len();
        let mut frame = [0u8; TOTAL_FRAME_SIZE];
        BigEndian::write_u32(&mut frame[..DATA_LEN_SIZE], chunk_length as u32);
        frame[DATA_LEN_SIZE..DATA_LEN_SIZE + chunk_length].copy_from_slice(chunk);
        sealed_frame[..frame.len()].copy_from_slice(&frame);

        aead::seal_in_place(
            &self.send_secret,
            &self.send_nonce.to_bytes(),
            &[0u8; 0],
            sealed_frame,
            TAG_SIZE,
        ).map_err(|e| err!(AuthCryptoError, "seal_in_place failed: {}", e))?;

        Ok(())
    }
}

impl<IoHandler> io::Read for SecretConnection<IoHandler>
where
    IoHandler: io::Read + io::Write + Send + Sync,
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
        self.io_handler.read_exact(&mut sealed_frame).unwrap();

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

        let chunk_length = BigEndian::read_u32(&chunk_length_specifier);
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

impl<IoHandler> io::Write for SecretConnection<IoHandler>
where
    IoHandler: io::Read + io::Write + Send + Sync,
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
fn gen_eph_keys() -> ([u8; 32], [u8; 32]) {
    let mut local_csprng = OsRng::new().unwrap();
    let local_privkey = generate_secret(&mut local_csprng);
    let local_pubkey = generate_public(&local_privkey);
    (local_pubkey.to_bytes(), local_privkey)
}

// Returns remote_eph_pubkey
fn share_eph_pubkey<IoHandler: io::Read + io::Write + Send + Sync>(
    handler: &mut IoHandler,
    local_eph_pubkey: &[u8; 32],
) -> Result<[u8; 32], ()> {
    // Send our pubkey and receive theirs in tandem.
    // TODO(ismail): on the go side this is done in parallel, here we do send and receive after
    // each other. thread::spawn would require a static lifetime.
    // Should still work though.

    let mut buf = vec![0; 0];
    let local_eph_pubkey_vec = &local_eph_pubkey.to_vec();
    // Note: this is not regular protobuf encoding but raw length prefixed amino encoding:
    encode_varint(local_eph_pubkey_vec.len() as u64, &mut buf);
    buf.put_slice(local_eph_pubkey_vec);
    // this is the sending part of:
    // https://github.com/tendermint/tendermint/blob/013b9cef642f875634c614019ab13b17570778ad/p2p/conn/secret_connection.go#L208-L238
    // TODO(ismail): handle error here! This currently would panic on failure:
    handler
        .write_all(&buf)
        .expect("couldn't share local key with peer");

    let mut buf = vec![0; 33];
    handler.read_exact(&mut buf).unwrap();

    let mut amino_buf = Cursor::new(buf);
    // this is the receiving part of:
    // https://github.com/tendermint/tendermint/blob/013b9cef642f875634c614019ab13b17570778ad/p2p/conn/secret_connection.go#L208-L238
    let mut remote_eph_pubkey = vec![];
    merge(
        WireType::LengthDelimited,
        &mut remote_eph_pubkey,
        &mut amino_buf,
    ).unwrap();
    let mut remote_eph_pubkey_fixed: [u8; 32] = Default::default();
    remote_eph_pubkey_fixed.copy_from_slice(&remote_eph_pubkey[..32]);

    Ok(remote_eph_pubkey_fixed)
}

// Returns recv secret, send secret, challenge as 32 byte arrays
fn derive_secrets_and_challenge(
    shared_secret: &[u8; 32],
    loc_is_lo: bool,
) -> ([u8; 32], [u8; 32], [u8; 32]) {
    let info = b"TENDERMINT_SECRET_CONNECTION_KEY_AND_CHALLENGE_GEN";
    let hk = Hkdf::<Sha256>::extract(None, shared_secret);
    let hkdf_vector = hk.expand(&*info, 96);

    let challenge_vector = &hkdf_vector[64..96];
    let mut challenge: [u8; 32] = [0; 32];
    challenge.copy_from_slice(challenge_vector);
    let mut recv_secret = [0u8; 32];
    let mut send_secret = [0u8; 32];
    if loc_is_lo {
        recv_secret.copy_from_slice(&hkdf_vector[0..32]);
        send_secret.copy_from_slice(&hkdf_vector[32..64]);
    } else {
        send_secret.copy_from_slice(&hkdf_vector[0..32]);
        recv_secret.copy_from_slice(&hkdf_vector[32..64]);
    }
    (recv_secret, send_secret, challenge)
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
fn sign_challenge(challenge: [u8; 32], local_privkey: &DalekSigner) -> Result<Signature, Error> {
    local_privkey
        .sign(&challenge)
        .map_err(|e| err!(SigningError, "{}", e))
}

#[derive(Clone, PartialEq, Message)]
struct AuthSigMessage {
    #[prost(bytes, tag = "1")]
    key: Vec<u8>,
    #[prost(bytes, tag = "2")]
    sig: Vec<u8>,
}

// TODO(ismail): change from DecodeError to something more generic
// this can also fail while writing / sending
fn share_auth_signature<IoHandler: io::Read + io::Write + Send + Sync>(
    sc: &mut SecretConnection<IoHandler>,
    pubkey: &[u8; 32],
    signature: Signature,
) -> Result<AuthSigMessage, DecodeError> {
    let amsg = AuthSigMessage {
        key: pubkey.to_vec(),
        sig: signature.into_bytes().to_vec(),
    };
    let mut buf: Vec<u8> = vec![];
    amsg.encode(&mut buf).unwrap();

    sc.write_all(&buf).unwrap();

    let mut rbuf = vec![0; 100]; // 100 = 32 + 64 + (amino overhead)
    sc.read_exact(&mut rbuf).unwrap();

    // TODO: proper error handling:
    Ok(AuthSigMessage::decode(&rbuf)?)
}

struct Nonce(pub [u8; 12]);

impl Default for Nonce {
    fn default() -> Nonce {
        Nonce([0u8; AEAD_NONCE_SIZE])
    }
}

impl Nonce {
    fn increment(&mut self) {
        let counter: u64 = BigEndian::read_u64(&self.0[4..]);
        BigEndian::write_u64(&mut self.0[4..], counter.checked_add(1).unwrap());
    }

    #[inline]
    fn to_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

#[cfg(test)]
mod tests {
    use secret_connection;
    use secret_connection::{Nonce, AEAD_NONCE_SIZE};
    use std::collections::HashMap;
    use x25519_dalek::diffie_hellman;

    #[test]
    fn test_incr_nonce() {
        // make sure we match the golang implementation
        let mut check_points: HashMap<i32, &[u8]> = HashMap::new();
        check_points.insert(0, &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        check_points.insert(1, &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        check_points.insert(510, &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255]);
        check_points.insert(511, &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0]);
        check_points.insert(512, &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1]);
        check_points.insert(1023, &[0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0]);

        let mut nonce = Nonce::default();
        assert_eq!(nonce.to_bytes().len(), AEAD_NONCE_SIZE);

        for i in 0..1024 {
            nonce.increment();
            match check_points.get(&i) {
                Some(want) => {
                    let got = &nonce.to_bytes();
                    assert_eq!(got, want);
                }
                None => (),
            }
        }
    }
    #[test]
    #[should_panic]
    fn test_incr_nonce_overflow() {
        // other than in the golang implementation we panic if we incremented more than 64
        // bits allow.
        // In golang this would reset to an all zeroes nonce.
        let mut nonce = Nonce([0u8, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255]);
        nonce.increment();
    }

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
        let (ref t3, ref t4) = secret_connection::sort32(t1, t2);
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

    #[test]
    fn test_derive_secrets_and_challenge_golden_test_vectors() {
        extern crate hex;
        use std::fs::File;
        use std::io::BufRead;
        use std::io::BufReader;
        use std::str::FromStr;

        let f = File::open("src/TestDeriveSecretsAndChallenge.golden").unwrap();
        let file = BufReader::new(&f);
        for line in file.lines() {
            let l = line.unwrap();
            let params: Vec<&str> = l.split(',').collect();

            let rand_secret_vector: Vec<u8> = hex::decode(params.get(0).unwrap()).unwrap();
            let mut rand_secret: [u8; 32] = [0x0; 32];
            rand_secret.copy_from_slice(&rand_secret_vector);

            let loc_is_least = bool::from_str(params.get(1).unwrap()).unwrap();
            let expected_recv_secret = hex::decode(params.get(2).unwrap()).unwrap();
            let expected_send_secret = hex::decode(params.get(3).unwrap()).unwrap();
            let expected_challenge = hex::decode(params.get(4).unwrap()).unwrap();
            let (recv_secret, send_secret, challenge) =
                secret_connection::derive_secrets_and_challenge(&rand_secret, loc_is_least);

            assert_eq!(
                expected_recv_secret, recv_secret,
                "Recv Secrets aren't equal"
            );
            assert_eq!(
                expected_send_secret, send_secret,
                "Send Secrets aren't equal"
            );
            assert_eq!(expected_challenge, challenge, "challenges aren't equal");
        }
    }
}
