use byteorder::{BigEndian, ByteOrder, LittleEndian};
use error::Error;
use hkdf::Hkdf;
use ring::aead;
use sha2::Sha256;

pub struct HkdfChaChaPoly {
    key: [u8; 32],
}

/// Provides aead operations
pub trait Aead {
    fn name(&self) -> &'static str;

    fn set(&mut self, key: [u8; 32]);
    fn seal(
        &self,
        nonce: &[u8; NONCE_SIZE],
        authtext: &[u8],
        plaintext: &[u8],
        out: &mut [u8],
    ) -> Result<usize, ()>;
    fn open(
        &self,
        nonce: &[u8; NONCE_SIZE],
        authtext: &[u8],
        ciphertext: &[u8],
        out: &mut [u8],
    ) -> Result<usize, ()>;
}

// KEY_SIZE is the size of the key used by this AEAD, in bytes.
pub const KEY_SIZE: usize = 32;
// NONCE_SIZE is the size of the nonce used with this AEAD, in bytes.
pub const NONCE_SIZE: usize = 24;
// CHACHA_NONCE_SIZE is the size of the nonce used in Chacha
pub const CHACHA_NONCE_SIZE: usize = 12;
// TAG_SIZE is the size added from poly1305
pub const TAG_SIZE: usize = 16;
// MAX_PLAINTEXT_SIZE is the max size that can be passed into a single call of Seal
pub const MAX_PLAINTEXT_SIZE: usize = (1 << 38) - 64;
// MAX_CIPHERTEXT_SIZE is the max size that can be passed into a single call of Open,
// this differs from plaintext size due to the tag
pub const MAX_CIPHERTEXT_SIZE: usize = MAX_PLAINTEXT_SIZE - 16;
// HKDF_INFO is the parameter used internally for Hkdf's info parameter.
const HKDF_INFO: &str = "TENDERMINT_SECRET_CONNECTION_FRAME_KEY_DERIVE";

impl Aead for HkdfChaChaPoly {
    fn name(&self) -> &'static str {
        "HkdfChaChaPoly"
    }

    fn set(&mut self, key: [u8; 32]) {
        self.key = key;
    }

    fn seal(
        &self,
        nonce: &[u8; NONCE_SIZE],
        authtext: &[u8],
        plaintext: &[u8],
        out: &mut [u8],
    ) -> Result<usize, ()> {
        out[..plaintext.len()].copy_from_slice(plaintext);
        // if plaintext.len() > MAX_PLAINTEXT_SIZE {
        //     return error!("Plaintext is greater than the maximum size")
        // }

        let (subkey, chacha_nonce) = get_subkey_and_chacha_nonce_from_hkdf(&self.key, &nonce);
        let sealing_key = aead::SealingKey::new(&aead::CHACHA20_POLY1305, &subkey).unwrap();
        let res = aead::seal_in_place(
            &sealing_key,
            &chacha_nonce,
            authtext,
            &mut out[..plaintext.len() + TAG_SIZE],
            16,
        );
        Ok(plaintext.len() + TAG_SIZE)
    }

    fn open(
        &self,
        nonce: &[u8; NONCE_SIZE],
        authtext: &[u8],
        ciphertext: &[u8],
        out: &mut [u8],
    ) -> Result<usize, ()> {
        // if ciphertext.len() > MAX_CIPHERTEXT_SIZE {
        //     return error!("Plaintext is greater than the maximum size")
        // }

        let (subkey, chacha_nonce) = get_subkey_and_chacha_nonce_from_hkdf(&self.key, &nonce);
        let opening_key = aead::OpeningKey::new(&aead::CHACHA20_POLY1305, &subkey).unwrap();
        // optimize if the provided buffer is sufficiently large
        if out.len() >= ciphertext.len() {
            let in_out = &mut out[..ciphertext.len()];
            in_out.copy_from_slice(ciphertext);

            let len = aead::open_in_place(&opening_key, &chacha_nonce, authtext, 0, in_out)
                .map_err(|_| ())?
                .len();

            Ok(len)
        } else {
            let mut in_out = ciphertext.to_vec();

            let out0 = aead::open_in_place(&opening_key, &chacha_nonce, authtext, 0, &mut in_out)
                .map_err(|_| ())?;
            out[..out0.len()].copy_from_slice(out0);
            Ok(out0.len())
        }
    }
}

pub fn new_hkdfchachapoly(key: [u8; 32]) -> HkdfChaChaPoly {
    return HkdfChaChaPoly { key: key };
}

// Returns subkey and chacha nonce
fn get_subkey_and_chacha_nonce_from_hkdf(
    c_key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
) -> ([u8; KEY_SIZE], [u8; CHACHA_NONCE_SIZE]) {
    let info = HKDF_INFO.as_bytes();

    let hk = Hkdf::<Sha256>::extract(Some(nonce), c_key);
    let subkey_and_nonce_vec = hk.expand(&info, KEY_SIZE + CHACHA_NONCE_SIZE);

    let subkey_vec = &subkey_and_nonce_vec[..KEY_SIZE];
    let chacha_nonce_vec = &subkey_and_nonce_vec[KEY_SIZE..];
    // Now convert vectors into fix sized byte arr
    let mut subkey: [u8; KEY_SIZE] = [0; KEY_SIZE];
    let subkey_vec = &subkey_vec[..KEY_SIZE]; // panics if not enough data
    subkey.copy_from_slice(subkey_vec);

    let mut chacha_nonce: [u8; CHACHA_NONCE_SIZE] = [0; CHACHA_NONCE_SIZE];
    let chacha_nonce_vec = &chacha_nonce_vec[..CHACHA_NONCE_SIZE]; // panics if not enough data
    chacha_nonce.copy_from_slice(chacha_nonce_vec);
    return (subkey, chacha_nonce);
}

#[cfg(test)]
mod tests {
    use hex;
    use hkdfchachapoly;
    use hkdfchachapoly::Aead;
    #[test]
    fn test_vector() {
        let key_vec = hex::decode(
            "56f8de45d3c294c7675bcaf457bdd4b71c380b9b2408ce9412b348d0f08b69ee",
        ).unwrap();
        // Now convert vector into fix sized byte arr
        let mut key: [u8; hkdfchachapoly::KEY_SIZE] = [0; hkdfchachapoly::KEY_SIZE];
        let key_vec = &key_vec[..hkdfchachapoly::KEY_SIZE]; // panics if not enough data
        key.copy_from_slice(key_vec);

        let aead_instance = hkdfchachapoly::HkdfChaChaPoly { key: key };

        let mut ciphertexts: [&str; 10] = [""; 10];
        ciphertexts[0] = "e20a8bf42c535ac30125cfc52031577f0b";
        ciphertexts[1] = "657695b37ba30f67b25860d90a6f1d00d8";
        ciphertexts[2] = "e9aa6f3b7f625d957fd50f05bcdf20d014";
        ciphertexts[3] = "8a00b3b5a6014e0d2033bebc5935086245";
        ciphertexts[4] = "aadd74867b923879e6866ea9e03c009039";
        ciphertexts[5] = "fc59773c2c864ee3b4cc971876b3c7bed4";
        ciphertexts[6] = "caec14e3a9a52ce1a2682c6737defa4752";
        ciphertexts[7] = "0b89511ffe490d2049d6950494ee51f919";
        ciphertexts[8] = "7de854ea71f43ca35167a07566c769083d";
        ciphertexts[9] = "cd477327f4ea4765c71e311c5fec1edbfb";

        for i in 0u8..10u8 {
            let ciphertext = decode_ciphertext(ciphertexts[i as usize]);
            let mut out = [0u8; 1];
            let mut byte_arr = [0u8; 1];
            byte_arr[0] = i;
            let mut nonce = [0u8; hkdfchachapoly::NONCE_SIZE];
            nonce[0] = byte_arr[0];

            let res = aead_instance.open(&nonce, &byte_arr, &ciphertext, &mut out);
            assert_eq!(res.unwrap(), 1);
            let mut expected = [0u8; 17];
            expected[0] = byte_arr[0];
            assert_eq!(out, byte_arr);
            let mut ct = [0u8; 17];
            aead_instance.seal(&nonce, &byte_arr, &byte_arr, &mut ct);
            assert_eq!(ct, ciphertext);
        }
    }

    fn decode_ciphertext(hex_msg: &str) -> [u8; 17] {
        let ct_vec = hex::decode(hex_msg).unwrap();
        // Now convert vector into fix sized byte arr
        let mut ciphertext: [u8; 17] = [0; 17];
        let ct_vec = &ct_vec[..17]; // panics if not enough data
        ciphertext.copy_from_slice(ct_vec);
        return ciphertext;
    }
}
