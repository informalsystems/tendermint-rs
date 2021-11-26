//! Proposals from validators

mod canonical_proposal;
mod msg_type;
mod sign_proposal;

pub use self::canonical_proposal::CanonicalProposal;
pub use msg_type::Type;
pub use sign_proposal::{SignProposalRequest, SignedProposalResponse};

use crate::block::{Height, Id as BlockId, Round};
use crate::chain::Id as ChainId;
use crate::consensus::State;
use crate::error::Error;
use crate::prelude::*;
use crate::Signature;
use crate::Time;
use bytes::BufMut;
use core::convert::{TryFrom, TryInto};
use tendermint_proto::types::Proposal as RawProposal;
use tendermint_proto::{Error as ProtobufError, Protobuf};

/// Proposal
#[derive(Clone, PartialEq, Debug)]
pub struct Proposal {
    /// Proposal message type
    pub msg_type: Type,
    /// Height
    pub height: Height,
    /// Round
    pub round: Round,
    /// POL Round
    pub pol_round: Option<Round>,
    /// Block ID
    pub block_id: Option<BlockId>,
    /// Timestamp
    pub timestamp: Option<Time>,
    /// Signature
    pub signature: Option<Signature>,
}

impl Protobuf<RawProposal> for Proposal {}

impl TryFrom<RawProposal> for Proposal {
    type Error = Error;

    fn try_from(value: RawProposal) -> Result<Self, Self::Error> {
        if value.pol_round < -1 {
            return Err(Error::negative_pol_round());
        }
        let pol_round = match value.pol_round {
            -1 => None,
            n => Some(Round::try_from(n)?),
        };
        Ok(Proposal {
            msg_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: value.round.try_into()?,
            pol_round,
            block_id: value.block_id.map(TryInto::try_into).transpose()?,
            timestamp: value.timestamp.map(|t| t.try_into()).transpose()?,
            signature: Signature::new(value.signature)?,
        })
    }
}

impl From<Proposal> for RawProposal {
    fn from(value: Proposal) -> Self {
        RawProposal {
            r#type: value.msg_type.into(),
            height: value.height.into(),
            round: value.round.into(),
            pol_round: value.pol_round.map_or(-1, Into::into),
            block_id: value.block_id.map(Into::into),
            timestamp: value.timestamp.map(Into::into),
            signature: value.signature.map(|s| s.to_bytes()).unwrap_or_default(),
        }
    }
}

impl Proposal {
    /// Create signable bytes from Proposal.
    pub fn to_signable_bytes<B>(
        &self,
        chain_id: ChainId,
        sign_bytes: &mut B,
    ) -> Result<bool, ProtobufError>
    where
        B: BufMut,
    {
        CanonicalProposal::new(self.clone(), chain_id).encode_length_delimited(sign_bytes)?;
        Ok(true)
    }

    /// Create signable vector from Proposal.
    pub fn to_signable_vec(&self, chain_id: ChainId) -> Result<Vec<u8>, ProtobufError> {
        CanonicalProposal::new(self.clone(), chain_id).encode_length_delimited_vec()
    }

    /// Consensus state from this proposal - This doesn't seem to be used anywhere.
    #[deprecated(
        since = "0.17.0",
        note = "This seems unnecessary, please raise it to the team, if you need it."
    )]
    pub fn consensus_state(&self) -> State {
        State {
            height: self.height,
            round: self.round,
            step: 3,
            block_id: self.block_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block::parts::Header;
    use crate::block::Id as BlockId;
    use crate::block::{Height, Round};
    use crate::chain::Id as ChainId;
    use crate::hash::{Algorithm, Hash};
    use crate::prelude::*;
    use crate::proposal::SignProposalRequest;
    use crate::test::dummy_signature;
    use crate::{proposal::Type, Proposal};
    use core::convert::TryInto;
    use core::str::FromStr;
    use tendermint_proto::Protobuf;
    use time::macros::datetime;

    #[test]
    fn test_serialization() {
        let dt = datetime!(2018-02-11 07:09:22.765 UTC);
        let proposal = Proposal {
            msg_type: Type::Proposal,
            height: Height::from(12345_u32),
            round: Round::from(23456_u16),
            pol_round: None,
            block_id: Some(BlockId {
                hash: Hash::from_hex_upper(
                    Algorithm::Sha256,
                    "DEADBEEFDEADBEEFBAFBAFBAFBAFBAFADEADBEEFDEADBEEFBAFBAFBAFBAFBAFA",
                )
                .unwrap(),
                part_set_header: Header::new(
                    65535,
                    Hash::from_hex_upper(
                        Algorithm::Sha256,
                        "0022446688AACCEE1133557799BBDDFF0022446688AACCEE1133557799BBDDFF",
                    )
                    .unwrap(),
                )
                .unwrap(),
            }),
            timestamp: Some(dt.try_into().unwrap()),
            signature: Some(dummy_signature()),
        };

        let mut got = vec![];

        let request = SignProposalRequest {
            proposal,
            chain_id: ChainId::from_str("test_chain_id").unwrap(),
        };

        let _have = request.to_signable_bytes(&mut got);

        // the following vector is generated via:
        /*
            import (
                "encoding/hex"
                "fmt"
                prototypes "github.com/tendermint/tendermint/proto/tendermint/types"
                "github.com/tendermint/tendermint/types"
                "strings"
                "time"
            )

            func proposalSerialize() {
                stamp, _ := time.Parse(time.RFC3339Nano, "2018-02-11T07:09:22.765Z")
                block_hash, _ := hex.DecodeString("DEADBEEFDEADBEEFBAFBAFBAFBAFBAFADEADBEEFDEADBEEFBAFBAFBAFBAFBAFA")
                part_hash, _ := hex.DecodeString("0022446688AACCEE1133557799BBDDFF0022446688AACCEE1133557799BBDDFF")
                proposal := &types.Proposal{
                    Type:     prototypes.SignedMsgType(prototypes.ProposalType),
                    Height:   12345,
                    Round:    23456,
                    POLRound: -1,
                    BlockID: types.BlockID{
                        Hash: block_hash,
                        PartSetHeader: types.PartSetHeader{
                            Hash:  part_hash,
                            Total: 65535,
                        },
                    },
                    Timestamp: stamp,
                }
                signBytes := types.ProposalSignBytes("test_chain_id", proposal.ToProto())
                fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v", signBytes), " "), ", "))
            }
        */

        let want = vec![
            136, 1, 8, 32, 17, 57, 48, 0, 0, 0, 0, 0, 0, 25, 160, 91, 0, 0, 0, 0, 0, 0, 32, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 1, 42, 74, 10, 32, 222, 173, 190, 239, 222,
            173, 190, 239, 186, 251, 175, 186, 251, 175, 186, 250, 222, 173, 190, 239, 222, 173,
            190, 239, 186, 251, 175, 186, 251, 175, 186, 250, 18, 38, 8, 255, 255, 3, 18, 32, 0,
            34, 68, 102, 136, 170, 204, 238, 17, 51, 85, 119, 153, 187, 221, 255, 0, 34, 68, 102,
            136, 170, 204, 238, 17, 51, 85, 119, 153, 187, 221, 255, 50, 12, 8, 162, 216, 255, 211,
            5, 16, 192, 242, 227, 236, 2, 58, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110,
            95, 105, 100,
        ];

        assert_eq!(got, want)
    }

    #[test]
    // Test proposal encoding with a malformed block ID which is considered null in Go.
    fn test_encoding_with_empty_block_id() {
        let dt = datetime!(2018-02-11 07:09:22.765 UTC);
        let proposal = Proposal {
            msg_type: Type::Proposal,
            height: Height::from(12345_u32),
            round: Round::from(23456_u16),
            pol_round: None,
            block_id: Some(BlockId {
                hash: Hash::from_hex_upper(Algorithm::Sha256, "").unwrap(),
                part_set_header: Header::new(
                    65535,
                    Hash::from_hex_upper(
                        Algorithm::Sha256,
                        "0022446688AACCEE1133557799BBDDFF0022446688AACCEE1133557799BBDDFF",
                    )
                    .unwrap(),
                )
                .unwrap(),
            }),
            timestamp: Some(dt.try_into().unwrap()),
            signature: Some(dummy_signature()),
        };

        let mut got = vec![];

        let request = SignProposalRequest {
            proposal,
            chain_id: ChainId::from_str("test_chain_id").unwrap(),
        };

        let _have = request.to_signable_bytes(&mut got);

        // the following vector is generated via:
        /*
            import (
                "encoding/hex"
                "fmt"
                prototypes "github.com/tendermint/tendermint/proto/tendermint/types"
                "github.com/tendermint/tendermint/types"
                "strings"
                "time"
            )

            func proposalSerialize() {
                stamp, _ := time.Parse(time.RFC3339Nano, "2018-02-11T07:09:22.765Z")
                block_hash, _ := hex.DecodeString("")
                part_hash, _ := hex.DecodeString("0022446688AACCEE1133557799BBDDFF0022446688AACCEE1133557799BBDDFF")
                proposal := &types.Proposal{
                    Type:     prototypes.SignedMsgType(prototypes.ProposalType),
                    Height:   12345,
                    Round:    23456,
                    POLRound: -1,
                    BlockID: types.BlockID{
                        Hash: block_hash,
                        PartSetHeader: types.PartSetHeader{
                            Hash:  part_hash,
                            Total: 65535,
                        },
                    },
                    Timestamp: stamp,
                }
                signBytes := types.ProposalSignBytes("test_chain_id", proposal.ToProto())
                fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v", signBytes), " "), ", "))
            }
        */

        let want = vec![
            102, 8, 32, 17, 57, 48, 0, 0, 0, 0, 0, 0, 25, 160, 91, 0, 0, 0, 0, 0, 0, 32, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 1, 42, 40, 18, 38, 8, 255, 255, 3, 18, 32, 0, 34,
            68, 102, 136, 170, 204, 238, 17, 51, 85, 119, 153, 187, 221, 255, 0, 34, 68, 102, 136,
            170, 204, 238, 17, 51, 85, 119, 153, 187, 221, 255, 50, 12, 8, 162, 216, 255, 211, 5,
            16, 192, 242, 227, 236, 2, 58, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95,
            105, 100,
        ];

        assert_eq!(got, want)
    }

    #[test]
    fn test_deserialization() {
        let dt = datetime!(2018-02-11 07:09:22.765 UTC);
        let proposal = Proposal {
            msg_type: Type::Proposal,
            height: Height::from(12345_u32),
            round: Round::from(23456_u16),
            timestamp: Some(dt.try_into().unwrap()),

            pol_round: None,
            block_id: Some(BlockId {
                hash: Hash::from_hex_upper(
                    Algorithm::Sha256,
                    "DEADBEEFDEADBEEFBAFBAFBAFBAFBAFADEADBEEFDEADBEEFBAFBAFBAFBAFBAFA",
                )
                .unwrap(),
                part_set_header: Header::new(
                    65535,
                    Hash::from_hex_upper(
                        Algorithm::Sha256,
                        "0022446688AACCEE1133557799BBDDFF0022446688AACCEE1133557799BBDDFF",
                    )
                    .unwrap(),
                )
                .unwrap(),
            }),
            signature: Some(dummy_signature()),
        };
        let want = SignProposalRequest {
            proposal,
            chain_id: ChainId::from_str("test_chain_id").unwrap(),
        };

        let data = vec![
            10, 176, 1, 8, 32, 16, 185, 96, 24, 160, 183, 1, 32, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 1, 42, 74, 10, 32, 222, 173, 190, 239, 222, 173, 190, 239, 186, 251, 175,
            186, 251, 175, 186, 250, 222, 173, 190, 239, 222, 173, 190, 239, 186, 251, 175, 186,
            251, 175, 186, 250, 18, 38, 8, 255, 255, 3, 18, 32, 0, 34, 68, 102, 136, 170, 204, 238,
            17, 51, 85, 119, 153, 187, 221, 255, 0, 34, 68, 102, 136, 170, 204, 238, 17, 51, 85,
            119, 153, 187, 221, 255, 50, 12, 8, 162, 216, 255, 211, 5, 16, 192, 242, 227, 236, 2,
            58, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95,
            105, 100,
        ];

        let have = SignProposalRequest::decode_vec(&data).unwrap();
        assert_eq!(have, want);
    }
}
