use crate::chain::Id as ChainId;
use crate::prelude::*;
use crate::Error;
use core::convert::TryFrom;
use tendermint_proto::privval::PubKeyRequest as RawPubKeyRequest;
use tendermint_proto::Protobuf;

/// PubKeyRequest requests the consensus public key from the remote signer.
#[derive(Clone, PartialEq, Debug)]
pub struct PubKeyRequest {
    /// Chain ID
    pub chain_id: ChainId,
}

impl Protobuf<RawPubKeyRequest> for PubKeyRequest {}

impl TryFrom<RawPubKeyRequest> for PubKeyRequest {
    type Error = Error;

    fn try_from(value: RawPubKeyRequest) -> Result<Self, Self::Error> {
        Ok(PubKeyRequest {
            chain_id: ChainId::try_from(value.chain_id)?,
        })
    }
}

impl From<PubKeyRequest> for RawPubKeyRequest {
    fn from(value: PubKeyRequest) -> Self {
        RawPubKeyRequest {
            chain_id: value.chain_id.as_str().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PubKeyRequest;
    use crate::chain::Id as ChainId;
    use crate::prelude::*;
    use core::str::FromStr;
    use tendermint_proto::Protobuf;

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
                   ChainId: "A",
               }
               pbpk, _ := pkr.Marshal()
               fmt.Printf("%#v\n", pbpk)

           }
        */

        let want: Vec<u8> = vec![10, 1, 65];
        let msg = PubKeyRequest {
            chain_id: ChainId::from_str("A").unwrap(),
        };
        let mut got = vec![];
        let _have = msg.encode(&mut got);

        assert_eq!(got, want);

        match PubKeyRequest::decode(want.as_ref()) {
            Ok(have) => assert_eq!(have, msg),
            Err(err) => panic!("{}", err.to_string()),
        }
    }
}
