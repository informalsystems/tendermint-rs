//! Amino types used by Secret Connection

use prost_amino_derive::Message;

/// Authentication signature message
#[derive(Clone, PartialEq, Message)]
pub struct AuthSigMessage {
    /// Public key
    #[prost_amino(bytes, tag = "1", amino_name = "tendermint/PubKeyEd25519")]
    pub pub_key: Vec<u8>,

    /// Signature
    #[prost_amino(bytes, tag = "2")]
    pub sig: Vec<u8>,
}
