//! Secret Connection Protocol: message framing and versioning

use std::convert::TryInto;

use ed25519_dalek as ed25519;
use prost::Message as _;

use x25519_dalek::PublicKey as EphemeralPublic;

use tendermint_proto as proto;

#[cfg(feature = "amino")]
use super::amino_types;

use crate::error::Error;

/// Size of an X25519 or Ed25519 public key
const PUBLIC_KEY_SIZE: usize = 32;

/// Protocol version (based on the Tendermint version)
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum Version {
    /// Tendermint v0.34
    V0_34,

    /// Tendermint v0.33
    V0_33,

    /// Pre-Tendermint v0.33
    Legacy,
}

impl Version {
    /// Does this version of Secret Connection use a transcript hash
    #[must_use]
    pub fn has_transcript(self) -> bool {
        self != Self::Legacy
    }

    /// Are messages encoded using Protocol Buffers?
    #[must_use]
    pub const fn is_protobuf(self) -> bool {
        match self {
            Self::V0_34 => true,
            Self::V0_33 | Self::Legacy => false,
        }
    }

    /// Encode the initial handshake message (i.e. first one sent by both peers)
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn encode_initial_handshake(self, eph_pubkey: &EphemeralPublic) -> Vec<u8> {
        if self.is_protobuf() {
            // Equivalent Go implementation:
            // https://github.com/tendermint/tendermint/blob/9e98c74/p2p/conn/secret_connection.go#L307-L312
            // TODO(tarcieri): proper protobuf framing
            let mut buf = Vec::new();
            buf.extend_from_slice(&[0x22, 0x0a, 0x20]);
            buf.extend_from_slice(eph_pubkey.as_bytes());
            buf
        } else {
            // Legacy Amino encoded handshake message
            // Equivalent Go implementation:
            // https://github.com/tendermint/tendermint/blob/013b9ce/p2p/conn/secret_connection.go#L213-L217
            //
            // Note: this is not regular protobuf encoding but raw length prefixed amino encoding;
            // amino prefixes with the total length, and the raw bytes array's length, too:
            let mut buf = vec![PUBLIC_KEY_SIZE as u8 + 1, PUBLIC_KEY_SIZE as u8];
            buf.extend_from_slice(eph_pubkey.as_bytes());
            buf
        }
    }

    /// Decode the initial handshake message
    ///
    /// # Errors
    ///
    /// * if the message is malformed
    pub fn decode_initial_handshake(self, bytes: &[u8]) -> Result<EphemeralPublic, Error> {
        let eph_pubkey = if self.is_protobuf() {
            // Equivalent Go implementation:
            // https://github.com/tendermint/tendermint/blob/9e98c74/p2p/conn/secret_connection.go#L315-L323
            // TODO(tarcieri): proper protobuf framing
            if bytes.len() != 34 || bytes[..2] != [0x0a, 0x20] {
                return Err(Error::malformed_handshake());
            }

            let eph_pubkey_bytes: [u8; 32] = bytes[2..].try_into().expect("framing failed");
            EphemeralPublic::from(eph_pubkey_bytes)
        } else {
            // Equivalent Go implementation:
            // https://github.com/tendermint/tendermint/blob/013b9ce/p2p/conn/secret_connection.go#L220-L225
            //
            // Check that the length matches what we expect and the length prefix is correct
            if bytes.len() != 33 || bytes[0] != 32 {
                return Err(Error::malformed_handshake());
            }

            let eph_pubkey_bytes: [u8; 32] = bytes[1..].try_into().expect("framing failed");
            EphemeralPublic::from(eph_pubkey_bytes)
        };

        // Reject the key if it is of low order
        if is_low_order_point(&eph_pubkey) {
            return Err(Error::low_order_key());
        }

        Ok(eph_pubkey)
    }

    /// Encode signature which authenticates the handshake
    #[must_use]
    pub fn encode_auth_signature(
        self,
        pub_key: &ed25519::PublicKey,
        signature: &ed25519::Signature,
    ) -> Vec<u8> {
        if self.is_protobuf() {
            // Protobuf `AuthSigMessage`
            let pub_key = proto::crypto::PublicKey {
                sum: Some(proto::crypto::public_key::Sum::Ed25519(
                    pub_key.as_ref().to_vec(),
                )),
            };

            let msg = proto::p2p::AuthSigMessage {
                pub_key: Some(pub_key),
                sig: signature.as_ref().to_vec(),
            };

            let mut buf = Vec::new();
            msg.encode_length_delimited(&mut buf)
                .expect("couldn't encode AuthSigMessage proto");
            buf
        } else {
            self.encode_auth_signature_amino(pub_key, signature)
        }
    }

    /// Get the length of the auth message response for this protocol version
    #[must_use]
    pub const fn auth_sig_msg_response_len(self) -> usize {
        if self.is_protobuf() {
            // 32 + 64 + (proto overhead = 1 prefix + 2 fields + 2 lengths + total length)
            103
        } else {
            // 32 + 64 + (amino overhead = 2 fields + 2 lengths + 4 prefix bytes + total length)
            106
        }
    }

    /// Decode signature message which authenticates the handshake
    ///
    /// # Errors
    ///
    /// * if the decoding of the bytes fails
    pub fn decode_auth_signature(self, bytes: &[u8]) -> Result<proto::p2p::AuthSigMessage, Error> {
        if self.is_protobuf() {
            // Parse Protobuf-encoded `AuthSigMessage`
            proto::p2p::AuthSigMessage::decode_length_delimited(bytes).map_err(Error::decode)
        } else {
            self.decode_auth_signature_amino(bytes)
        }
    }

    #[allow(clippy::unused_self)]
    #[cfg(feature = "amino")]
    fn encode_auth_signature_amino(
        self,
        pub_key: &ed25519::PublicKey,
        signature: &ed25519::Signature,
    ) -> Vec<u8> {
        // Legacy Amino encoded `AuthSigMessage`
        let msg = amino_types::AuthSigMessage::new(pub_key, signature);

        let mut buf = Vec::new();
        msg.encode_length_delimited(&mut buf)
            .expect("encode_auth_signature failed");
        buf
    }

    #[allow(clippy::unused_self)]
    #[cfg(not(feature = "amino"))]
    fn encode_auth_signature_amino(
        self,
        _: &ed25519::PublicKey,
        _: &ed25519::Signature,
    ) -> Vec<u8> {
        panic!("attempted to encode auth signature using amino, but 'amino' feature is not present")
    }

    #[allow(clippy::unused_self)]
    #[cfg(feature = "amino")]
    fn decode_auth_signature_amino(
        self,
        bytes: &[u8],
    ) -> Result<proto::p2p::AuthSigMessage, Error> {
        // Legacy Amino encoded `AuthSigMessage`
        let amino_msg =
            amino_types::AuthSigMessage::decode_length_delimited(bytes).map_err(Error::decode)?;

        amino_msg.try_into()
    }

    #[allow(clippy::unused_self)]
    #[cfg(not(feature = "amino"))]
    fn decode_auth_signature_amino(self, _: &[u8]) -> Result<proto::p2p::AuthSigMessage, Error> {
        panic!("attempted to decode auth signature using amino, but 'amino' feature is not present")
    }
}

/// Reject low order points listed on <https://cr.yp.to/ecdh.html>
///
/// These points contain low-order X25519 field elements. Rejecting them is
/// suggested in the "May the Fourth" paper under Section 5:
/// Software Countermeasures (see "Rejecting Known Bad Points" subsection):
///
/// <https://eprint.iacr.org/2017/806.pdf>
#[allow(clippy::match_same_arms)]
fn is_low_order_point(point: &EphemeralPublic) -> bool {
    // Note: as these are public points and do not interact with secret-key
    // material in any way, this check does not need to be performed in
    // constant-time.
    match point.as_bytes() {
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

#[cfg(all(test, feature = "amino"))]
mod tests {
    use super::{ed25519, Version};
    use core::convert::TryFrom;

    #[test]
    fn encode_auth_signature_amino() {
        let pub_key = ed25519::PublicKey::from_bytes(&[
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64,
            0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68,
            0xf7, 0x07, 0x51, 0x1a,
        ])
        .unwrap();

        let sig = ed25519::Signature::try_from(
            [
                0xe5, 0x56, 0x43, 0x00, 0xc3, 0x60, 0xac, 0x72, 0x90, 0x86, 0xe2, 0xcc, 0x80, 0x6e,
                0x82, 0x8a, 0x84, 0x87, 0x7f, 0x1e, 0xb8, 0xe5, 0xd9, 0x74, 0xd8, 0x73, 0xe0, 0x65,
                0x22, 0x49, 0x01, 0x55, 0x5f, 0xb8, 0x82, 0x15, 0x90, 0xa3, 0x3b, 0xac, 0xc6, 0x1e,
                0x39, 0x70, 0x1c, 0xf9, 0xb4, 0x6b, 0xd2, 0x5b, 0xf5, 0xf0, 0x59, 0x5b, 0xbe, 0x24,
                0x65, 0x51, 0x41, 0x43, 0x8e, 0x7a, 0x10, 0x0b,
            ]
            .as_ref(),
        )
        .unwrap();

        let actual_msg = Version::Legacy.encode_auth_signature_amino(&pub_key, &sig);
        let expected_msg = [
            105, 10, 37, 22, 36, 222, 100, 32, 215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254,
            211, 201, 100, 7, 58, 14, 225, 114, 243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81,
            26, 18, 64, 229, 86, 67, 0, 195, 96, 172, 114, 144, 134, 226, 204, 128, 110, 130, 138,
            132, 135, 127, 30, 184, 229, 217, 116, 216, 115, 224, 101, 34, 73, 1, 85, 95, 184, 130,
            21, 144, 163, 59, 172, 198, 30, 57, 112, 28, 249, 180, 107, 210, 91, 245, 240, 89, 91,
            190, 36, 101, 81, 65, 67, 142, 122, 16, 11,
        ];

        assert_eq!(expected_msg.as_ref(), actual_msg.as_slice());

        let decoded_msg = Version::Legacy
            .decode_auth_signature_amino(&actual_msg)
            .unwrap();

        match decoded_msg.pub_key.as_ref().unwrap().sum {
            Some(tendermint_proto::crypto::public_key::Sum::Ed25519(ref pk)) => {
                assert_eq!(pk, pub_key.as_bytes());
            }
            ref other => panic!("unexpected public key: {:?}", other),
        }

        assert_eq!(decoded_msg.sig, sig.as_ref());
    }
}
