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
    const fn encode_auth_signature_amino(
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
    const fn decode_auth_signature_amino(
        self,
        _: &[u8],
    ) -> Result<proto::p2p::AuthSigMessage, Error> {
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
