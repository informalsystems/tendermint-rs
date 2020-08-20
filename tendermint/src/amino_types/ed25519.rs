use crate::{
    error,
    public_key::{Ed25519, PublicKey},
    Error,
};
use anomaly::format_err;
use std::convert::TryFrom;
use tendermint_proto::crypto::public_key::Sum;

// Note:On the golang side this is generic in the sense that it could everything that implements
// github.com/tendermint/tendermint/crypto.PubKey
// While this is meant to be used with different key-types, it currently only uses a PubKeyEd25519
// version.
// TODO(ismail): make this more generic (by modifying prost and adding a trait for PubKey)

// Copied from tendermint_proto::privval::PubKeyResponse;
/// PubKeyResponse is a response message containing the public key.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyResponse {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::std::option::Option<tendermint_proto::crypto::PublicKey>,
    #[prost(message, optional, tag = "2")]
    pub error: ::std::option::Option<tendermint_proto::privval::RemoteSignerError>,
}

// Copied from tendermint_proto::privval::PubKeyRequest;
/// PubKeyRequest requests the consensus public key from the remote signer.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyRequest {
    #[prost(string, tag = "1")]
    pub chain_id: String,
}

impl TryFrom<PubKeyResponse> for PublicKey {
    type Error = Error;

    // This does not check if the underlying pub_key_ed25519 has the right size.
    // The caller needs to make sure that this is actually the case.
    fn try_from(response: PubKeyResponse) -> Result<PublicKey, Error> {
        match &response
            .pub_key
            .ok_or_else(|| format_err!(error::Kind::InvalidKey, "empty pubkey"))?
            .sum
            .ok_or_else(|| format_err!(error::Kind::InvalidKey, "empty sum"))?
        {
            Sum::Ed25519(b) => Ed25519::from_bytes(b),
        }
        .map(Into::into)
        .map_err(|_| format_err!(error::Kind::InvalidKey, "malformed key").into())
    }
}

impl From<PublicKey> for PubKeyResponse {
    fn from(public_key: PublicKey) -> PubKeyResponse {
        match public_key {
            PublicKey::Ed25519(ref pk) => PubKeyResponse {
                pub_key: Some(tendermint_proto::crypto::PublicKey {
                    sum: Some(tendermint_proto::crypto::public_key::Sum::Ed25519(
                        pk.as_bytes().to_vec(),
                    )),
                }),
                error: None,
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
    use prost::Message;
    use std::convert::TryInto;

    #[test]
    fn test_empty_pubkey_msg() {
        // test-vector generated via the following go code:
        /*
           import (
               "fmt"
               "github.com/tendermint/tendermint/proto/tendermint/privval"
           )
           func ed25519_empty() {
               pkr := &privval.PubKeyRequest{
                   ChainId: "",
               }
               pbpk, _ := pkr.Marshal()
               fmt.Printf("%#v\n", pbpk)

           }
        */

        let want: Vec<u8> = vec![];
        let msg = PubKeyRequest {
            chain_id: "".to_string(),
        };
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
        // test-vector generated from Go
        /*
           import (
               "fmt"
               "github.com/tendermint/tendermint/proto/tendermint/crypto"
               "github.com/tendermint/tendermint/proto/tendermint/privval"
           )

           func ed25519_key() {
               pkr := &privval.PubKeyResponse{
                   PubKey: &crypto.PublicKey{
                       Sum: &crypto.PublicKey_Ed25519{Ed25519: []byte{
                           0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0,
                           0x8b, 0x7b, 0xb5, 0x61, 0xbc, 0xe7, 0xc1, 0xd4, 0x69,
                           0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72, 0xa, 0x52,
                           0x4d, 0x37, 0x32, 0xef, 0xed,
                       },
                       },
                   },
                   Error: nil,
               }
               pbpk, _ := pkr.Marshal()
               fmt.Printf("%#v\n", pbpk)

           }
        */
        let encoded = vec![
            0xa, 0x22, 0xa, 0x20, 0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0, 0x8b, 0x7b,
            0xb5, 0x61, 0xbc, 0xe7, 0xc1, 0xd4, 0x69, 0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72,
            0xa, 0x52, 0x4d, 0x37, 0x32, 0xef, 0xed,
        ];

        let msg = PubKeyResponse {
            pub_key: Some(tendermint_proto::crypto::PublicKey {
                sum: Some(tendermint_proto::crypto::public_key::Sum::Ed25519(vec![
                    0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0, 0x8b, 0x7b, 0xb5, 0x61,
                    0xbc, 0xe7, 0xc1, 0xd4, 0x69, 0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72, 0xa,
                    0x52, 0x4d, 0x37, 0x32, 0xef, 0xed,
                ])),
            }),
            error: None,
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
            pub_key: Some(tendermint_proto::crypto::PublicKey {
                sum: Some(tendermint_proto::crypto::public_key::Sum::Ed25519(vec![
                    0xaf, 0xf3, 0x94, 0xc5, 0xb7, 0x5c, 0xfb, 0xd, 0xd9, 0x28, 0xe5, 0x8a, 0x92,
                    0xdd, 0x76, 0x55, 0x2b, 0x2e, 0x8d, 0x19, 0x6f, 0xe9, 0x12, 0x14, 0x50, 0x80,
                    0x6b, 0xd0, 0xd9, 0x3f, 0xd0, 0xcb,
                ])),
            }),
            error: None,
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
            pub_key: None,
            error: None,
        };
        // we expect this to panic:
        let _got: PublicKey = empty_msg.try_into().unwrap();
    }
}
