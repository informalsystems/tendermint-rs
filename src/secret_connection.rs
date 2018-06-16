#[allow(dead_code)]
mod conn {
    use hkdf::Hkdf;
    use sha2::Sha256;
    use ed25519::{Signer};
    use signatory::ed25519::Signature;
    use error::Error;
    use std::io;

    // 4 + 1024 == 1028 total frame size
    const DATA_LEN_SIZE: u32 = 4;
    const DATA_MAX_SIZE: u32 = 1024;
    const TOTAL_FRAME_SIZE: u32 = DATA_MAX_SIZE + DATA_LEN_SIZE;
    // 16 is the size of the mac tag
    const SEALED_FRAME_SIZE: u32 = TOTAL_FRAME_SIZE + 16;

    // // Implements net.Conn
    struct SecretConnection<IoHandler: io::Read + io::Write>  {
    	conn:       IoHandler,
    	recv_nonce:  [u8; 24],
    	send_nonce:  [u8; 24],
    	// remote_pubkey   crypto.PubKey,
    	shared_secret:  [u8; 32], // shared secret
    	recv_buffer: [u8],
    }
    //
    // // Performs handshake and returns a new authenticated SecretConnection.
    // // Returns nil if error in handshake.
    // // Caller should call conn.Close()
    // // See docs/sts-final.pdf for more information.
    // func MakeSecretConnection(conn io.ReadWriteCloser, locPrivKey crypto.PrivKey) (*SecretConnection, error) {
    //
    // 	locPubKey := locPrivKey.PubKey()
    //
    // 	// Generate ephemeral keys for perfect forward secrecy.
    // 	locEphPub, locEphPriv := genEphKeys()
    //
    // 	// Write local ephemeral pubkey and receive one too.
    // 	// NOTE: every 32-byte string is accepted as a Curve25519 public key
    // 	// (see DJB's Curve25519 paper: http://cr.yp.to/ecdh/curve25519-20060209.pdf)
    // 	remEphPub, err := shareEphPubKey(conn, locEphPub)
    // 	if err != nil {
    // 		return nil, err
    // 	}
    //
    // 	// Compute common shared secret.
    // 	shrSecret := computeSharedSecret(remEphPub, locEphPriv)
    //
    // 	// Sort by lexical order.
    // 	loEphPub, hiEphPub := sort32(locEphPub, remEphPub)
    //
    // 	// Check if the local ephemeral public key
    // 	// was the least, lexicographically sorted.
    // 	locIsLeast := bytes.Equal(locEphPub[:], loEphPub[:])
    //
    // 	// Generate nonces to use for secretbox.
    // 	recvNonce, sendNonce := genNonces(loEphPub, hiEphPub, locIsLeast)
    //
    // 	// Generate common challenge to sign.
    // 	challenge := genChallenge(loEphPub, hiEphPub)
    //
    // 	// Construct SecretConnection.
    // 	sc := &SecretConnection{
    // 		conn:       conn,
    // 		recvBuffer: nil,
    // 		recvNonce:  recvNonce,
    // 		sendNonce:  sendNonce,
    // 		shrSecret:  shrSecret,
    // 	}
    //
    // 	// Sign the challenge bytes for authentication.
    // 	locSignature := signChallenge(challenge, locPrivKey)
    //
    // 	// Share (in secret) each other's pubkey & challenge signature
    // 	authSigMsg, err := shareAuthSignature(sc, locPubKey, locSignature)
    // 	if err != nil {
    // 		return nil, err
    // 	}
    // 	remPubKey, remSignature := authSigMsg.Key, authSigMsg.Sig
    // 	if !remPubKey.VerifyBytes(challenge[:], remSignature) {
    // 		return nil, errors.New("Challenge verification failed")
    // 	}
    //
    // 	// We've authorized.
    // 	sc.remPubKey = remPubKey
    // 	return sc, nil
    // }
    //
    // // Returns authenticated remote pubkey
    // func (sc *SecretConnection) RemotePubKey() crypto.PubKey {
    // 	return sc.remPubKey
    // }
    //
    // // Writes encrypted frames of `sealedFrameSize`
    // // CONTRACT: data smaller than dataMaxSize is read atomically.
    // func (sc *SecretConnection) Write(data []byte) (n int, err error) {
    // 	for 0 < len(data) {
    // 		var frame = make([]byte, totalFrameSize)
    // 		var chunk []byte
    // 		if dataMaxSize < len(data) {
    // 			chunk = data[:dataMaxSize]
    // 			data = data[dataMaxSize:]
    // 		} else {
    // 			chunk = data
    // 			data = nil
    // 		}
    // 		chunkLength := len(chunk)
    // 		binary.BigEndian.PutUint32(frame, uint32(chunkLength))
    // 		copy(frame[dataLenSize:], chunk)
    //
    // 		aead, err := xchacha20poly1305.New(sc.shrSecret[:])
    // 		if err != nil {
    // 			return n, errors.New("Invalid SecretConnection Key")
    // 		}
    // 		// encrypt the frame
    // 		var sealedFrame = make([]byte, aead.Overhead()+totalFrameSize)
    // 		aead.Seal(sealedFrame[:0], sc.sendNonce[:], frame, nil)
    // 		// fmt.Printf("secretbox.Seal(sealed:%X,sendNonce:%X,shrSecret:%X\n", sealedFrame, sc.sendNonce, sc.shrSecret)
    // 		incr2Nonce(sc.sendNonce)
    // 		// end encryption
    //
    // 		_, err = sc.conn.Write(sealedFrame)
    // 		if err != nil {
    // 			return n, err
    // 		}
    // 		n += len(chunk)
    // 	}
    // 	return
    // }
    //
    // // CONTRACT: data smaller than dataMaxSize is read atomically.
    // func (sc *SecretConnection) Read(data []byte) (n int, err error) {
    // 	if 0 < len(sc.recvBuffer) {
    // 		n = copy(data, sc.recvBuffer)
    // 		sc.recvBuffer = sc.recvBuffer[n:]
    // 		return
    // 	}
    //
    // 	aead, err := xchacha20poly1305.New(sc.shrSecret[:])
    // 	if err != nil {
    // 		return n, errors.New("Invalid SecretConnection Key")
    // 	}
    // 	sealedFrame := make([]byte, totalFrameSize+aead.Overhead())
    // 	_, err = io.ReadFull(sc.conn, sealedFrame)
    // 	if err != nil {
    // 		return
    // 	}
    //
    // 	// decrypt the frame
    // 	var frame = make([]byte, totalFrameSize)
    // 	// fmt.Printf("secretbox.Open(sealed:%X,recvNonce:%X,shrSecret:%X\n", sealedFrame, sc.recvNonce, sc.shrSecret)
    // 	_, err = aead.Open(frame[:0], sc.recvNonce[:], sealedFrame, nil)
    // 	if err != nil {
    // 		return n, errors.New("Failed to decrypt SecretConnection")
    // 	}
    // 	incr2Nonce(sc.recvNonce)
    // 	// end decryption
    //
    // 	var chunkLength = binary.BigEndian.Uint32(frame) // read the first two bytes
    // 	if chunkLength > dataMaxSize {
    // 		return 0, errors.New("chunkLength is greater than dataMaxSize")
    // 	}
    // 	var chunk = frame[dataLenSize : dataLenSize+chunkLength]
    //
    // 	n = copy(data, chunk)
    // 	sc.recvBuffer = chunk[n:]
    // 	return
    // }
    //
    // // Implements net.Conn
    // func (sc *SecretConnection) Close() error                  { return sc.conn.Close() }
    // func (sc *SecretConnection) LocalAddr() net.Addr           { return sc.conn.(net.Conn).LocalAddr() }
    // func (sc *SecretConnection) RemoteAddr() net.Addr          { return sc.conn.(net.Conn).RemoteAddr() }
    // func (sc *SecretConnection) SetDeadline(t time.Time) error { return sc.conn.(net.Conn).SetDeadline(t) }
    // func (sc *SecretConnection) SetReadDeadline(t time.Time) error {
    // 	return sc.conn.(net.Conn).SetReadDeadline(t)
    // }
    // func (sc *SecretConnection) SetWriteDeadline(t time.Time) error {
    // 	return sc.conn.(net.Conn).SetWriteDeadline(t)
    // }

    // func genEphKeys() (ephPub, ephPriv *[32]byte) {
    // 	var err error
    // 	ephPub, ephPriv, err = box.GenerateKey(crand.Reader)
    // 	if err != nil {
    // 		panic("Could not generate ephemeral keypairs")
    // 	}
    // 	return
    // }

    // func shareEphPubKey(conn io.ReadWriteCloser, locEphPub *[32]byte) (remEphPub *[32]byte, err error) {
    //
    // 	// Send our pubkey and receive theirs in tandem.
    // 	var trs, _ = cmn.Parallel(
    // 		func(_ int) (val interface{}, err error, abort bool) {
    // 			var _, err1 = cdc.MarshalBinaryWriter(conn, locEphPub)
    // 			if err1 != nil {
    // 				return nil, err1, true // abort
    // 			} else {
    // 				return nil, nil, false
    // 			}
    // 		},
    // 		func(_ int) (val interface{}, err error, abort bool) {
    // 			var _remEphPub [32]byte
    // 			var _, err2 = cdc.UnmarshalBinaryReader(conn, &_remEphPub, 1024*1024) // TODO
    // 			if err2 != nil {
    // 				return nil, err2, true // abort
    // 			} else {
    // 				return _remEphPub, nil, false
    // 			}
    // 		},
    // 	)
    //
    // 	// If error:
    // 	if trs.FirstError() != nil {
    // 		err = trs.FirstError()
    // 		return
    // 	}
    //
    // 	// Otherwise:
    // 	var _remEphPub = trs.FirstValue().([32]byte)
    // 	return &_remEphPub, nil
    // }

    // Returns shared secret as 32 byte array
    fn compute_shared_secret(remote_eph_pubkey: [u8; 32], local_eph_privkey: [u8; 32]) -> [u8; 32] {
        let mut shared_key: [u8; 32] = [0; 32];
        let mut shared_secret: [u8; 32] = [0; 32];

        // TODO: Do DH to get shared key
        // curve25519.ScalarMult(sharedKey, privateKey, peersPublicKey);

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
    	return local_privkey.sign(&challenge[0..32])
    }
    //
    // type authSigMessage struct {
    // 	Key crypto.PubKey
    // 	Sig crypto.Signature
    // }
    //
    // fn shareAuthSignature(sc *SecretConnection, pubKey crypto.PubKey, signature crypto.Signature) (recvMsg authSigMessage, err error) {
    //
    // 	// Send our info and receive theirs in tandem.
    // 	var trs, _ = cmn.Parallel(
    // 		func(_ int) (val interface{}, err error, abort bool) {
    // 			var _, err1 = cdc.MarshalBinaryWriter(sc, authSigMessage{pubKey, signature})
    // 			if err1 != nil {
    // 				return nil, err1, true // abort
    // 			} else {
    // 				return nil, nil, false
    // 			}
    // 		},
    // 		func(_ int) (val interface{}, err error, abort bool) {
    // 			var _recvMsg authSigMessage
    // 			var _, err2 = cdc.UnmarshalBinaryReader(sc, &_recvMsg, 1024*1024) // TODO
    // 			if err2 != nil {
    // 				return nil, err2, true // abort
    // 			} else {
    // 				return _recvMsg, nil, false
    // 			}
    // 		},
    // 	)
    //
    // 	// If error:
    // 	if trs.FirstError() != nil {
    // 		err = trs.FirstError()
    // 		return
    // 	}
    //
    // 	var _recvMsg = trs.FirstValue().(authSigMessage)
    // 	return _recvMsg, nil
    // }

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
        use secret_connection::conn;

        #[test]
        fn incr2_nonce() {
            // TODO: Create test vectors for this instead of just printing the result.
            // conn::incr2_nonce(&mut x);
        }

        #[test]
        fn test_sort() {
            // sanity check
            let t1 = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ];
            let t2 = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1,
            ];
            let (ref t3, ref t4) = conn::sort32(t1, t2);
            assert_eq!(t1, *t3);
            assert_eq!(t2, *t4);
        }

        #[test]
        fn test_hash32() {
            // Single test vector created against go implementation
            let t = conn::hash32(&[0, 0, 0, 0]);
            let expected: [u8; 32] = [
                20, 4, 134, 42, 238, 181, 232, 222, 228, 231, 42, 153, 251, 130, 165, 55, 53, 121,
                78, 134, 189, 245, 251, 252, 129, 73, 2, 52, 163, 111, 7, 71,
            ];
            assert_eq!(t, expected);
        }

        #[test]
        fn test_hash24() {
            // Single test vector created against go implementation
            let t = conn::hash24(&[0, 0, 0, 0]);
            let expected: [u8; 24] = [
                201, 60, 46, 37, 116, 170, 172, 244, 248, 110, 1, 142, 64, 194, 90, 157, 98, 143,
                226, 116, 219, 55, 115, 243,
            ];
            assert_eq!(t, expected);
        }
    }
}
