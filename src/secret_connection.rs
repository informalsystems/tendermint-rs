use amino::*;
use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut};
use ed25519::{Signer};
use error::Error;
#[allow(dead_code)]
use hkdf::Hkdf;
use hkdfchachapoly::{new_hkdfchachapoly, Aead, TAG_SIZE};
use rand::OsRng;
use sha2::Sha256;
use signatory::ed25519::{Signature, PublicKey, Verifier, DefaultVerifier};
use std::io::Cursor;
use std::marker::{Send, Sync};
use std::{cmp, io};
use x25519_dalek::{diffie_hellman, generate_public, generate_secret};

// 4 + 1024 == 1028 total frame size
const DATA_LEN_SIZE: u32 = 4;
const DATA_MAX_SIZE: u32 = 1024;
const TOTAL_FRAME_SIZE: u32 = DATA_MAX_SIZE + DATA_LEN_SIZE;
// 16 is the size of the mac tag
const SEALED_FRAME_SIZE: u32 = TOTAL_FRAME_SIZE + TAG_SIZE as u32;

// Implements net.Conn
// TODO: Fix errors due to the last element not being constant size
struct SecretConnection<IoHandler: io::Read + io::Write + Send + Sync> {
    io_handler: IoHandler,
    recv_nonce: [u8; 24],
    send_nonce: [u8; 24],
    remote_pubkey: [u8; 32],
    shared_secret: [u8; 32], // shared secret
    recv_buffer: [u8; 1024],
}

// TODO: Test read/write
impl<IoHandler: io::Read + io::Write + Send + Sync>
    SecretConnection<IoHandler>
{
    // Returns authenticated remote pubkey
    fn remote_pubkey(&self) -> [u8; 32] {
        self.remote_pubkey
    }

    // Writes encrypted frames of `sealedFrameSize`
    // CONTRACT: data smaller than dataMaxSize is read atomically.
    fn write(&mut self, data: &[u8]) -> Result<usize, ()> {
        let mut n = 0usize;
        let mut data_copy = &data[..];
        while 0 < data_copy.len() {
            let mut frame = [0u8; TOTAL_FRAME_SIZE as usize];
            let mut chunk: &[u8];
            if DATA_MAX_SIZE < (data.len() as u32) {
                chunk = &data[..(DATA_MAX_SIZE as usize)];
                data_copy = &data_copy[(DATA_MAX_SIZE as usize)..];
            } else {
                chunk = data_copy;
                data_copy = &[0u8; 0];
            }
            let chunkLength = chunk.len();
            BigEndian::write_u32_into(&[chunkLength as u32], &mut frame[..8]);
            frame[(DATA_LEN_SIZE as usize)..].copy_from_slice(chunk);

            let aead = new_hkdfchachapoly(self.shared_secret);
            // encrypt the frame

            let mut sealedFrame = [0u8; TAG_SIZE + (TOTAL_FRAME_SIZE as usize)];
            aead.seal(&self.send_nonce, &[0u8; 0], &frame, &mut sealedFrame);
            incr2_nonce(&mut self.send_nonce);
            // end encryption

            self.io_handler.write(&sealedFrame);
            n = n + chunk.len();
        }
        return Ok(n);
    }

    // CONTRACT: data smaller than dataMaxSize is read atomically.
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ()> {
        let mut n = 0usize;
        if 0 < self.recv_buffer.len() {
            n = cmp::min(data.len(), self.recv_buffer.len());
            data.copy_from_slice(&self.recv_buffer[..n]);
            let mut leftover_portion = vec![0; self.recv_buffer.len() - n];
            leftover_portion.clone_from_slice(&self.recv_buffer[n..]);
            self.recv_buffer.clone_from_slice(&leftover_portion);
            return Ok(n);
        }

        let aead = new_hkdfchachapoly(self.shared_secret);
        let mut sealedFrame = [0u8; TAG_SIZE + (TOTAL_FRAME_SIZE as usize)];
        self.io_handler.read_exact(&mut sealedFrame);

        // decrypt the frame
        let mut frame = [0u8; TOTAL_FRAME_SIZE as usize];
        let res = aead.open(&self.send_nonce, &[0u8; 0], &sealedFrame, &mut frame);
        let mut frame_copy = [0u8; TOTAL_FRAME_SIZE as usize];
        frame_copy.clone_from_slice(&frame);
        if res.is_err() {
            return res;
        }
        incr2_nonce(&mut self.send_nonce);
        // end decryption

        let mut chunk_length_specifier = vec![0; 2];
        chunk_length_specifier.clone_from_slice(&frame[..2]);

        let chunk_length = BigEndian::read_u32(&chunk_length_specifier);
        if chunk_length > DATA_MAX_SIZE {
            // TODO: Err should say "chunk_length is greater than dataMaxSize", confused as to how to do this
            return Err(());
        } else {
            let mut chunk = vec![0; chunk_length as usize];
            chunk.clone_from_slice(
                &frame_copy
                    [(DATA_LEN_SIZE as usize)..(DATA_LEN_SIZE as usize + chunk_length as usize)],
            );
            n = cmp::min(data.len(), chunk.len());
            data.copy_from_slice(&chunk[..n]);
            self.recv_buffer.copy_from_slice(&chunk[n..]);
            return Ok(n);
        }
    }

	// Performs handshake and returns a new authenticated SecretConnection.
	fn new(
	    mut handler: IoHandler,
	    local_privkey: Signer,
	) -> Result<SecretConnection<IoHandler>, ()> {
		// TODO: Error check
	    let local_pubkey = local_privkey.public_key().unwrap();

	    // Generate ephemeral keys for perfect forward secrecy.
	    let (local_eph_pubkey, local_eph_privkey) = gen_eph_keys();

	    // Write local ephemeral pubkey and receive one too.
	    // NOTE: every 32-byte string is accepted as a Curve25519 public key
	    // (see DJB's Curve25519 paper: http://cr.yp.to/ecdh/curve25519-20060209.pdf)
	    let remote_eph_pubkey = share_eph_pubkey(&mut handler, &local_eph_pubkey).unwrap();

	    // Compute common shared secret.
	    let shared_secret = compute_shared_secret(&remote_eph_pubkey, &local_eph_privkey);

	    // Sort by lexical order.
	    let (low_eph_pubkey, high_eph_pubkey) = sort32(local_eph_pubkey, remote_eph_pubkey);

	    // Check if the local ephemeral public key
	    // was the least, lexicographically sorted.
	    let locIsLeast = (local_eph_pubkey == low_eph_pubkey);

	    // Generate nonces to use for secretbox.
	    let (recv_nonce, send_nonce) = gen_nonces(local_eph_pubkey, high_eph_pubkey, locIsLeast);

	    // Generate common challenge to sign.
	    let challenge = gen_challenge(local_eph_pubkey, high_eph_pubkey);

	    // Construct SecretConnection.
	    let mut sc = SecretConnection {
	        io_handler: handler,
	        recv_buffer: [0u8; 1024],
	        recv_nonce: recv_nonce,
	        send_nonce: send_nonce,
	        shared_secret: shared_secret,
	        remote_pubkey: remote_eph_pubkey,
	    };

	    // Sign the challenge bytes for authentication.
		// TODO: Error check
	    let local_signature = sign_challenge(challenge, local_privkey).unwrap();

	    // Share (in secret) each other's pubkey & challenge signature
		// TODO: Error check
	    let auth_sig_msg = share_auth_signature(&mut sc, local_pubkey.as_bytes(), local_signature).unwrap();

		let remote_pubkey = PublicKey::from_bytes(&auth_sig_msg.Key).unwrap();
		let remote_signature: &[u8] = &auth_sig_msg.Sig;
		let remote_sig = Signature::from_bytes(remote_signature).unwrap();

		let valid_sig = DefaultVerifier::verify(&remote_pubkey, &challenge, &remote_sig);
		if valid_sig.is_err() {
			// TODO: Add error message "Challenge verification failed"
			return Err(());
		}
	    // We've authorized.
	    sc.remote_pubkey = auth_sig_msg.Key;
	    return Ok(sc);
	}
}

//
// // Implements net.Conn
// func (sc *SecretConnection) Close() error                  { return sc.conn.Close() }
// func (sc *SecretConnection) SetDeadline(t time.Time) error { return sc.conn.(net.Conn).SetDeadline(t) }
// func (sc *SecretConnection) SetReadDeadline(t time.Time) error {
// 	return sc.conn.(net.Conn).SetReadDeadline(t)
// }
// func (sc *SecretConnection) SetWriteDeadline(t time.Time) error {
// 	return sc.conn.(net.Conn).SetWriteDeadline(t)
// }

// Returns pubkey, private key
fn gen_eph_keys() -> ([u8; 32], [u8; 32]) {
    let mut local_csprng = OsRng::new().unwrap();
    let local_privkey = generate_secret(&mut local_csprng);
    let local_pubkey = generate_public(&local_privkey);
    return (local_pubkey.to_bytes(), local_privkey);
}

// Returns remote_eph_pubkey
// TODO: Ask if this is the correct way to have the readers and writers in threads
fn share_eph_pubkey<IoHandler: io::Read +io::Write + Send + Sync>(
    handler: &mut IoHandler,
    local_eph_pubkey: &[u8; 32],
) -> Result<[u8; 32], ()> {
    // Send our pubkey and receive theirs in tandem.
    let mut buf = vec![0; 0];
    amino_bytes::encode(local_eph_pubkey, &mut buf);
    handler.write(&buf);

    let mut buf = vec![];
    handler.read(&mut buf);
    let mut amino_buf = Cursor::new(buf);

    // TODO: Add error checking here
    // Don't need output of this
    let remote_eph_pubkey_vec = amino_bytes::decode(&mut amino_buf).unwrap();
    // move this vector into a fixed size array
    let mut remote_eph_pubkey = [0u8; 32];
    let remote_eph_pubkey_vec = &remote_eph_pubkey_vec[..32]; // panics if not enough data
    remote_eph_pubkey.copy_from_slice(remote_eph_pubkey_vec);
    return Ok(remote_eph_pubkey);
}

// Returns shared secret as 32 byte array
fn compute_shared_secret(remote_eph_pubkey: &[u8; 32], local_eph_privkey: &[u8; 32]) -> [u8; 32] {
    let shared_key = diffie_hellman(local_eph_privkey, remote_eph_pubkey);

    let salt = "".as_bytes();
    let info = "TENDERMINT_SECRET_CONNECTION_SHARED_SECRET_GEN".as_bytes();

    let hk = Hkdf::<Sha256>::extract(Some(salt), &shared_key);
    let shared_secret_vector = hk.expand(&info, 32);
    // Now convert res_vector into fix sized 32 byte u8 arr
    let mut shared_secret: [u8; 32] = [0; 32];
    let shared_secret_vector = &shared_secret_vector[..shared_secret.len()]; // panics if not enough data
    shared_secret.copy_from_slice(shared_secret_vector);
    return shared_secret;
}

// Return is of the form lo, hi
fn sort32(foo: [u8; 32], bar: [u8; 32]) -> ([u8; 32], [u8; 32]) {
    if bar > foo {
        return (foo, bar);
    } else {
        return (bar, foo);
    }
}

// Returns recvNonce, sendNonce
fn gen_nonces(lo_pubkey: [u8; 32], hi_pubkey: [u8; 32], loc_is_lo: bool) -> ([u8; 24], [u8; 24]) {
    let mut aggregated_pubkey: [u8; 64] = [0; 64];
    aggregated_pubkey[0..32].copy_from_slice(&lo_pubkey[0..32]);
    aggregated_pubkey[32..64].copy_from_slice(&hi_pubkey[0..32]);

    let nonce1 = hash24(&aggregated_pubkey);
    let mut nonce2: [u8; 24] = [0; 24];
    nonce2.copy_from_slice(&nonce1[0..24]);
    nonce2[23] = nonce2[23] ^ 1;
    let recv_nonce: [u8; 24];
    let send_nonce: [u8; 24];
    if loc_is_lo {
        recv_nonce = nonce1;
        send_nonce = nonce2;
    } else {
        recv_nonce = nonce2;
        send_nonce = nonce1;
    }
    return (recv_nonce, send_nonce);
}

// Returns 32 byte challenge
fn gen_challenge(lo_pubkey: [u8; 32], hi_pubkey: [u8; 32]) -> [u8; 32] {
    let mut aggregated_pubkey: [u8; 64] = [0; 64];
    aggregated_pubkey[0..32].copy_from_slice(&lo_pubkey[0..32]);
    aggregated_pubkey[32..64].copy_from_slice(&hi_pubkey[0..32]);
    return hash32(&aggregated_pubkey);
}

// Sign the challenge with the local private key
fn sign_challenge(challenge: [u8; 32], local_privkey: Signer) -> Result<Signature, Error> {
    return local_privkey.sign(&challenge[0..32]);
}

struct auth_sig_message {
    Key: [u8; 32],
    Sig: [u8; 64],
}

// TODO: Test if this works, I have no idea what the encoding is doing underneath.
impl Amino for auth_sig_message {
    fn deserialize(data: &[u8]) -> Result<auth_sig_message, DecodeError> {
        let mut buf = Cursor::new(data);
        consume_length(&mut buf)?;

        check_field_number_typ3(1, Typ3Byte::Typ3_ByteLength, &mut buf)?;
        let key_vec = amino_bytes::decode(&mut buf)?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_vec);

        check_field_number_typ3(2, Typ3Byte::Typ3_ByteLength, &mut buf)?;
        let sig_vec = amino_bytes::decode(&mut buf)?;
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&sig_vec);

        Ok(auth_sig_message { Key: key, Sig: sig })
    }

    fn serialize(self) -> Vec<u8> {
        let mut buf = vec![];

        // encode the Validator Address
        encode_field_number_typ3(1, Typ3Byte::Typ3_Struct, &mut buf);
        {
            // encode the Key
            encode_field_number_typ3(1, Typ3Byte::Typ3_ByteLength, &mut buf);
            amino_bytes::encode(&self.Key, &mut buf);
            // encode the Key
            encode_field_number_typ3(2, Typ3Byte::Typ3_ByteLength, &mut buf);
            amino_bytes::encode(&self.Sig, &mut buf);
        }
        buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));

        let mut length_buf = vec![];
        encode_uvarint(buf.len() as u64, &mut length_buf);
        length_buf.append(&mut buf);

        return length_buf;
    }
}

fn share_auth_signature<IoHandler: io::Read + io::Write + Send + Sync>(
    mut sc: &mut SecretConnection<IoHandler>,
    pubkey: &[u8; 32],
    signature: Signature,
) -> Result<auth_sig_message, DecodeError> {
    let amsg = auth_sig_message {
        Key: *pubkey,
        Sig: signature.into_bytes(),
    };
    // TODO: Figure out how to amino decode/encode this struct, check errors
    sc.io_handler.write(&amsg.serialize());
    let mut buf = vec![];
    sc.io_handler.read(&mut buf);
    auth_sig_message::deserialize(&buf)
}

fn hash32(input: &[u8]) -> [u8; 32] {
    let salt = "".as_bytes();
    let info = "TENDERMINT_SECRET_CONNECTION_KEY_GEN".as_bytes();

    let hk = Hkdf::<Sha256>::extract(Some(salt), input);
    let res_vector = hk.expand(&info, 32);
    // Now convert res_vector into fix sized 32 byte u8 arr
    let mut res: [u8; 32] = [0; 32];
    let res_vector = &res_vector[..res.len()]; // panics if not enough data
    res.copy_from_slice(res_vector);
    return res;
}

fn hash24(input: &[u8]) -> [u8; 24] {
    let salt = "".as_bytes();
    let info = "TENDERMINT_SECRET_CONNECTION_NONCE_GEN".as_bytes();

    let hk = Hkdf::<Sha256>::extract(Some(salt), input);
    let res_vector = hk.expand(&info, 24);
    // Now convert res_vector into fix sized 24 byte u8 arr
    let mut res: [u8; 24] = [0; 24];
    let res_vector = &res_vector[..res.len()]; // panics if not enough data
    res.copy_from_slice(res_vector);
    return res;
}

// increment nonce big-endian by 2 with wraparound.
fn incr2_nonce(nonce: &mut [u8; 24]) {
    incr_nonce(nonce);
    incr_nonce(nonce);
}

// TODO: Check if internal representation is big or small endian
// increment nonce big-endian by 2 with wraparound.
fn incr_nonce(nonce: &mut [u8; 24]) {
    for i in (0..24).rev() {
        nonce[i] = nonce[i] + 1;
        if nonce[i] != 0 {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use secret_connection;
    #[test]
    fn incr2_nonce() {
        // TODO: Create test vectors for this instead of just printing the result.
        // conn::incr2_nonce(&mut x);
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
    fn test_hash32() {
        // Single test vector created against go implementation
        let t = secret_connection::hash32(&[0, 0, 0, 0]);
        let expected: [u8; 32] = [
            20, 4, 134, 42, 238, 181, 232, 222, 228, 231, 42, 153, 251, 130, 165, 55, 53, 121, 78,
            134, 189, 245, 251, 252, 129, 73, 2, 52, 163, 111, 7, 71,
        ];
        assert_eq!(t, expected);
    }

    #[test]
    fn test_hash24() {
        // Single test vector created against go implementation
        let t = secret_connection::hash24(&[0, 0, 0, 0]);
        let expected: [u8; 24] = [
            201, 60, 46, 37, 116, 170, 172, 244, 248, 110, 1, 142, 64, 194, 90, 157, 98, 143, 226,
            116, 219, 55, 115, 243,
        ];
        assert_eq!(t, expected);
    }
}
