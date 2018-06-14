#[allow(dead_code)]
mod conn {
    use rust_sodium::crypto::secretbox::{NONCEBYTES, MACBYTES};
    use rust_sodium::randombytes::randombytes_into;
    use sha2::{Sha256, Digest as sha256_digest};
    use ripemd160::{Ripemd160, Digest as ripemd_digest};


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
    fn gen_challenge(loPubKey:[u8; 32], hiPubKey:[u8; 32]) -> [u8; 32] {
        let mut aggregated_pubkey: [u8; 64] = [0; 64];
        aggregated_pubkey[0..32].copy_from_slice(&loPubKey[0..32]);
        aggregated_pubkey[32..64].copy_from_slice(&hiPubKey[0..32]);
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
    pub fn gen_nonce() -> [u8; 24] {
        let mut nonce = [0; NONCEBYTES];
        randombytes_into(&mut nonce);
        return nonce;
    }

    // sha256
    fn hash32(input:&[u8]) -> [u8; 32] {
	    let mut sh = Sha256::default();
        sh.input(input);
        let res_slice = sh.result();
        let mut res: [u8; 32] = Default::default();
        res.copy_from_slice(&res_slice[0..32]);
        return res;
    }

    // We only fill in the first 20 bytes with ripemd160
    fn hash24(input:&[u8]) -> [u8; 24] {
        let mut rmd = Ripemd160::default();
        rmd.input(input);
        let res_slice = rmd.result();
        let mut res: [u8; 24] = Default::default();
        res[..20].copy_from_slice(&res_slice[0..20]);
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
            let mut x = conn::gen_nonce();
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
        fn sha2() {
            // Single test vector created with python's hashlib
            let t = conn::hash32(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
            let expected : [u8; 32] = [102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32, 8, 151, 20, 133, 110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41, 37];
            assert_eq!(t, expected);
        }

        #[test]
        fn ripemd160() {
            // Single test vector created with python's hashlib
            let t = conn::hash24(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
            let expected : [u8; 24] = [249, 36, 109, 210, 219, 4, 0, 89, 203, 207, 170, 22, 60, 54, 71, 150, 205, 171, 220, 146,0,0,0,0];
            assert_eq!(t, expected);
        }
    }
}
