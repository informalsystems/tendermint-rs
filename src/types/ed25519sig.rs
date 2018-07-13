// Note:On the golang side this is generic in the sense that it could everything that implements
// github.com/tendermint/tendermint/crypto.PubKey
// While this is meant to be used with different key-types, it currently only uses a PubKeyEd25519
// version.
// TODO(ismail): make this more generic (by modifying prost and adding a trait for PubKey)
#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/socketpv/PubKeyMsg"]
pub struct PubKeyMsg {
    #[prost(message, tag = "1")]
    pub_key_ed25519: Option<Vec<u8>>,
}

// TODO
//#[derive(Copy, Clone, PartialEq, Message)]
//pub struct PubKeyEd25519(pub [u8; PUBLIC_KEY_SIZE]);

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

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
        //	cdc.RegisterConcrete(&privval.PubKeyMsg{},
        //      "tendermint/socketpv/PubKeyMsg", nil)
        //	b, _ := cdc.MarshalBinary(&privval.PubKeyMsg{})
        //	fmt.Printf("%#v\n\n", b)
        //}
        // --------------------------------------------------------------------
        // Output:
        // []byte{0x4, 0x82, 0x7b, 0xe, 0x9e}
        //
        //

        let want = vec![0x4, 0x82, 0x7b, 0xe, 0x9e];
        let msg = PubKeyMsg { pub_key_ed25519: None };
        let mut got = vec![];
        let _have = msg.encode(&mut got);

        assert_eq!(got, want);
    }

    #[test]
    fn test_ed25519_pubkey_msg() {
        // test-vector generated exactly as for test_empty_pubkey_msg
        // but with the following modifications:
        //  var pubKey [32]byte
        //	copy(pubKey[:],[]byte{0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0, 0x8b, 0x7b,
        //  0xb5, 0x61, 0xbc, 0xe7, 0xc1,
        //	0xd4, 0x69, 0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72, 0xa, 0x52, 0x4d, 0x37, 0x32, 0xef, 0xed})
        //
        //	b, _ = cdc.MarshalBinary(&privval.PubKeyMsg{PubKey: crypto.PubKeyEd25519(pubKey)})
        //	fmt.Printf("%#v\n\n", b)
        //
        let want = vec![0x2b, 0x82, 0x7b, 0xe, 0x9e, 0xa, 0x25, 0x16, 0x24,
                        0xde, 0x64, 0x20,
                        // raw pub key bytes:
                        0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a,
                        0xec, 0xe0, 0x8b, 0x7b, 0xb5, 0x61, 0xbc, 0xe7, 0xc1, 0xd4,
                        0x69, 0xc3, 0x44, 0x26, 0xec, 0xef, 0xc0, 0x72, 0xa, 0x52,
                        0x4d, 0x37, 0x32, 0xef, 0xed];

        let msg = PubKeyMsg {
            pub_key_ed25519: Some(vec![0x79, 0xce, 0xd, 0xe0, 0x43, 0x33, 0x4a, 0xec, 0xe0, 0x8b,
                                       0x7b, 0xb5, 0x61, 0xbc, 0xe7, 0xc1, 0xd4, 0x69, 0xc3, 0x44,
                                       0x26, 0xec, 0xef, 0xc0, 0x72, 0xa, 0x52, 0x4d, 0x37, 0x32,
                                       0xef, 0xed])
        };

        let mut got = vec![];
        let _have = msg.encode(&mut got);

        assert_eq!(got, want);
    }
}