#[allow(dead_code)]
mod conn {
    use rust_sodium::crypto::secretbox::{NONCEBYTES, MACBYTES};
    use rust_sodium::randombytes::randombytes_into;
    use sha2::Sha256;
    use hkdf::Hkdf;

    // 4 + 1024 == 1028 total frame size
    const DATA_LEN_SIZE: u32 = 4;
    const DATA_MAX_SIZE: u32 = 1024;
    const TOTAL_FRAME_SIZE: u32 = DATA_MAX_SIZE + DATA_LEN_SIZE;
    const SEALED_FRAME_SIZE: u32 = TOTAL_FRAME_SIZE + (MACBYTES as u32);


    // Return is of the form lo, hi
    fn sort32(foo:[u8; 32], bar:[u8; 32]) -> ([u8; 32], [u8; 32]) {
    	if bar > foo {
    		return (foo, bar);
    	} else {
    		return (bar, foo);
    	}
    }

    // Returns recvNonce, sendNonce
    fn gen_nonces(loPubKey:[u8; 32], hiPubKey:[u8; 32], locIsLo:bool) -> ([u8; 24], [u8; 24]) {
        let mut aggregated_pubkey: [u8; 64] = [0; 64];
        aggregated_pubkey[0..32].copy_from_slice(&loPubKey[0..32]);
        aggregated_pubkey[32..64].copy_from_slice(&hiPubKey[0..32]);

    	let nonce1 = hash24(&aggregated_pubkey);
    	let mut nonce2: [u8; 24] = [0; 24];
        nonce2.copy_from_slice(&nonce1[0..24]);
    	nonce2[23] = nonce2[23] ^ 1;
        let recv_nonce: [u8; 24];
        let send_nonce: [u8; 24];
    	if locIsLo {
    		recv_nonce = nonce1;
    		send_nonce = nonce2;
    	} else {
    		recv_nonce = nonce2;
    		send_nonce = nonce1;
    	}
    	return (recv_nonce, send_nonce)
    }

    // Returns 32 byte challenge
    fn gen_challenge(lo_pubkey:[u8; 32], hi_pubkey:[u8; 32]) -> [u8; 32] {
        let mut aggregated_pubkey: [u8; 64] = [0; 64];
        aggregated_pubkey[0..32].copy_from_slice(&lo_pubkey[0..32]);
        aggregated_pubkey[32..64].copy_from_slice(&hi_pubkey[0..32]);
    	return hash32(&aggregated_pubkey)
    }
    //
    // func signChallenge(challenge *[32]byte, locPrivKey crypto.PrivKey) (signature crypto.Signature) {
    // 	signature = locPrivKey.Sign(challenge[:])
    // 	return
    // }
    //
    // type authSigMessage struct {
    // 	Key crypto.PubKey
    // 	Sig crypto.Signature
    // }
    //
    // func shareAuthSignature(sc *SecretConnection, pubKey crypto.PubKey, signature crypto.Signature) (recvMsg authSigMessage, err error) {
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


    // Basically copied from rust_sodium, because I was having issues with their wrapper Nonce type
    pub fn gen_rand_nonce() -> [u8; 24] {
        let mut nonce = [0; NONCEBYTES];
        randombytes_into(&mut nonce);
        return nonce;
    }

    fn hash32(input:&[u8]) -> [u8; 32] {
        let salt = "".as_bytes();
        let info = "TENDERMINT_SECRET_CONNECTION_KEY_GEN".as_bytes();

        let hk = Hkdf::<Sha256>::extract(Some(salt), input);
        let res_vector = hk.expand(&info, 32);
        // Now convert res_vector into fix sized 32 byte u8 arr
        let mut res:[u8; 32] = [0; 32];
        let res_vector = &res_vector[..res.len()]; // panics if not enough data
        res.copy_from_slice(res_vector);
        return res;
    }

    fn hash24(input:&[u8]) -> [u8; 24] {
        let salt = "".as_bytes();
        let info = "TENDERMINT_SECRET_CONNECTION_NONCE_GEN".as_bytes();

        let hk = Hkdf::<Sha256>::extract(Some(salt), input);
        let res_vector = hk.expand(&info, 24);
        // Now convert res_vector into fix sized 24 byte u8 arr
        let mut res:[u8; 24] = [0; 24];
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
    fn incr_nonce(nonce:&mut [u8; 24]) {
        for i in (0..24).rev() {
    		nonce[i] = nonce[i] + 1;
    		if nonce[i] != 0 {
    			return;
    		}
        }
    }

    #[cfg(test)]
    mod tests{
        use rust_sodium::crypto::secretbox::{Nonce, MACBYTES};
        use ::secret_connection::conn;

        #[test]
        fn incr2_nonce() {
            // TODO: Create test vectors instead of just printing the result.
            let mut x = conn::gen_rand_nonce();
            conn::incr2_nonce(&mut x);
        }

        #[test]
        fn test_sort() {
            // sanity check
            let t1 = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
            let t2 = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1];
            let (ref t3, ref t4) = conn::sort32(t1,t2);
            assert_eq!(t1, *t3);
        }

        #[test]
        fn test_hash32() {
            // Single test vector created against go implementation
            let t = conn::hash32(&[0,0,0,0]);
            let expected : [u8; 32] = [20, 4, 134, 42, 238, 181, 232, 222, 228, 231, 42, 153, 251, 130, 165, 55, 53, 121, 78, 134, 189, 245, 251, 252, 129, 73, 2, 52, 163, 111, 7, 71];
            assert_eq!(t, expected);
        }

        #[test]
        fn test_hash24() {
            // Single test vector created against go implementation
            let t = conn::hash24(&[0,0,0,0]);
            let expected : [u8; 24] = [201, 60, 46, 37, 116, 170, 172, 244, 248, 110, 1, 142, 64, 194, 90, 157, 98, 143, 226, 116, 219, 55, 115, 243];
            assert_eq!(t, expected);
        }
    }
}
