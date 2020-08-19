use super::compute_prefix;
use crate::{
    error,
    public_key::{Ed25519, PublicKey},
    Error,
};
use anomaly::format_err;
use once_cell::sync::Lazy;
use prost_amino_derive::Message;
use std::convert::TryFrom;

// Note:On the golang side this is generic in the sense that it could everything that implements
// github.com/tendermint/tendermint/crypto.PubKey
// While this is meant to be used with different key-types, it currently only uses a PubKeyEd25519
// version.
// TODO(ismail): make this more generic (by modifying prost and adding a trait for PubKey)

pub const AMINO_NAME: &str = "tendermint/remotesigner/PubKeyRequest";
pub static AMINO_PREFIX: Lazy<Vec<u8>> = Lazy::new(|| compute_prefix(AMINO_NAME));

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/PubKeyResponse"]
pub struct PubKeyResponse {
    #[prost_amino(bytes, tag = "1", amino_name = "tendermint/PubKeyEd25519")]
    pub pub_key_ed25519: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/PubKeyRequest"]
pub struct PubKeyRequest {}

impl TryFrom<PubKeyResponse> for PublicKey {
    type Error = Error;

    // This does not check if the underlying pub_key_ed25519 has the right size.
    // The caller needs to make sure that this is actually the case.
    fn try_from(response: PubKeyResponse) -> Result<PublicKey, Error> {
        Ed25519::from_bytes(&response.pub_key_ed25519)
            .map(Into::into)
            .map_err(|_| format_err!(error::Kind::InvalidKey, "malformed Ed25519 key").into())
    }
}

impl From<PublicKey> for PubKeyResponse {
    fn from(public_key: PublicKey) -> PubKeyResponse {
        match public_key {
            PublicKey::Ed25519(ref pk) => PubKeyResponse {
                pub_key_ed25519: pk.as_bytes().to_vec(),
            },
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(_) => panic!("secp256k1 PubKeyResponse unimplemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::PUBLIC_KEY_LENGTH;
    use prost_amino::Message;
    use std::convert::TryInto;

    #[test]
    fn test_empty_pubkey_msg() {
        // test-vector generated via the following go code:
        //
        // --------------------------------------------------------------------
        //package main
        //
        //import (
        //	"fmt"
        //
        //	"github.com/tendermint/go-amino"
        //	"github.com/tendermint/tendermint/crypto"
        //	"github.com/tendermint/tendermint/privval"
        //)
        //
        //func main() {
        //	cdc := amino.NewCodec()
        //
        //	cdc.RegisterInterface((*crypto.PubKey)(nil), nil)
        //	cdc.RegisterConcrete(crypto.PubKeyEd25519{},
        //		"tendermint/PubKeyEd25519", nil)
        //	cdc.RegisterConcrete(&privval.PubKeyRequest{},
        //      "tendermint/remotesigner/PubKeyRequest", nil)
        //	b, _ := cdc.MarshalBinary(&privval.PubKeyRequest{})
        //	fmt.Printf("%#v\n\n", b)
        //}
        // --------------------------------------------------------------------
        // Output:
        // []byte{0x4, 0xcb, 0x94, 0xd6, 0x20}
        //
        //

        let want = vec![0x4, 0xcb, 0x94, 0xd6, 0x20];
        let msg = PubKeyRequest {};
        let mut got = vec![];
        let _have = msg.encode(&mut got);

        assert_eq!(got, want);

        match PubKeyRequest::decode(want.as_ref()) {
            Ok(have) => assert_eq!(have, msg),
            Err(err) => panic!(err.to_string()),
        }
    }

    #[test]
    fn test_ed25519_pubkey_msg() {
        // test-vector generated exactly as for test_empty_pubkey_msg
        // but with the following modifications:
        //	cdc.RegisterConcrete(&privval.PubKeyResponse{},
        //      "tendermint/remotesigner/PubKeyResponse", nil)
        //
        //  var pubKey [32]byte
        //	copy(pubKey[:],[]byte{0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0, 0x8b, 0x7b,
        //  0xb5, 0x61, 0xbc, 0xe7, 0xc1,
        //	0xd4, 0x69, 0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72, 0xa, 0x52, 0x4d, 0x37, 0x32, 0xef,
        // 0xed})
        //
        //	b, _ = cdc.MarshalBinary(&privval.PubKeyResponse{PubKey: crypto.PubKeyEd25519(pubKey)})
        //	fmt.Printf("%#v\n\n", b)
        //
        let encoded = vec![
            0x2b, // length
            0x17, 0xe, 0xd5, 0x7c, // prefix
            0xa, 0x25, 0x16, 0x24, 0xde, 0x64, 0x20, 0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec,
            0xe0, 0x8b, 0x7b, 0xb5, 0x61, 0xbc, 0xe7, 0xc1, 0xd4, 0x69, 0xc3, 0x44, 0x26, 0xec,
            0xef, 0xc0, 0x72, 0xa, 0x52, 0x4d, 0x37, 0x32, 0xef, 0xed,
        ];

        let msg = PubKeyResponse {
            pub_key_ed25519: vec![
                0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0, 0x8b, 0x7b, 0xb5, 0x61, 0xbc,
                0xe7, 0xc1, 0xd4, 0x69, 0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72, 0xa, 0x52, 0x4d,
                0x37, 0x32, 0xef, 0xed,
            ],
        };
        let mut got = vec![];
        let _have = msg.encode(&mut got);

        assert_eq!(got, encoded);

        match PubKeyResponse::decode(encoded.as_ref()) {
            Ok(have) => assert_eq!(have, msg),
            Err(err) => panic!(err),
        }
    }

    #[test]
    fn test_into() {
        let raw_pk: [u8; PUBLIC_KEY_LENGTH] = [
            0xaf, 0xf3, 0x94, 0xc5, 0xb7, 0x5c, 0xfb, 0xd, 0xd9, 0x28, 0xe5, 0x8a, 0x92, 0xdd,
            0x76, 0x55, 0x2b, 0x2e, 0x8d, 0x19, 0x6f, 0xe9, 0x12, 0x14, 0x50, 0x80, 0x6b, 0xd0,
            0xd9, 0x3f, 0xd0, 0xcb,
        ];
        let want = PublicKey::Ed25519(Ed25519::from_bytes(&raw_pk).unwrap());
        let pk = PubKeyResponse {
            pub_key_ed25519: vec![
                0xaf, 0xf3, 0x94, 0xc5, 0xb7, 0x5c, 0xfb, 0xd, 0xd9, 0x28, 0xe5, 0x8a, 0x92, 0xdd,
                0x76, 0x55, 0x2b, 0x2e, 0x8d, 0x19, 0x6f, 0xe9, 0x12, 0x14, 0x50, 0x80, 0x6b, 0xd0,
                0xd9, 0x3f, 0xd0, 0xcb,
            ],
        };
        let orig = pk.clone();
        let got: PublicKey = pk.try_into().unwrap();

        assert_eq!(got, want);

        // and back:
        let round_trip_pk: PubKeyResponse = got.into();
        assert_eq!(round_trip_pk, orig);
    }

    #[test]
    #[should_panic]
    fn test_empty_into() {
        let empty_msg = PubKeyResponse {
            pub_key_ed25519: vec![],
        };
        // we expect this to panic:
        let _got: PublicKey = empty_msg.try_into().unwrap();
    }
}
