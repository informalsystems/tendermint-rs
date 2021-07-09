#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSet {
    #[prost(message, repeated, tag="1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(message, optional, tag="2")]
    pub proposer: ::core::option::Option<Validator>,
    #[prost(int64, tag="3")]
    pub total_voting_power: i64,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    #[prost(bytes="vec", tag="1")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag="3")]
    #[serde(alias = "power", with = "crate::serializers::from_str")]
    pub voting_power: i64,
    #[prost(int64, tag="4")]
    #[serde(with = "crate::serializers::from_str", default)]
    pub proposer_priority: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleValidator {
    #[prost(message, optional, tag="1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag="2")]
    pub voting_power: i64,
}
/// PartsetHeader
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartSetHeader {
    #[prost(uint32, tag="1")]
    #[serde(with = "crate::serializers::part_set_header_total")]
    pub total: u32,
    #[prost(bytes="vec", tag="2")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Part {
    #[prost(uint32, tag="1")]
    pub index: u32,
    #[prost(bytes="vec", tag="2")]
    pub bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="3")]
    pub proof: ::core::option::Option<super::crypto::Proof>,
}
/// BlockID
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(bytes="vec", tag="1")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    #[serde(alias = "parts")]
    pub part_set_header: ::core::option::Option<PartSetHeader>,
}
// --------------------------------

/// Header defines the structure of a Tendermint block header.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// basic block info
    #[prost(message, optional, tag="1")]
    pub version: ::core::option::Option<super::version::Consensus>,
    #[prost(string, tag="2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag="3")]
    #[serde(with = "crate::serializers::from_str")]
    pub height: i64,
    #[prost(message, optional, tag="4")]
    #[serde(with = "crate::serializers::optional")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// prev block info
    #[prost(message, optional, tag="5")]
    pub last_block_id: ::core::option::Option<BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[prost(bytes="vec", tag="6")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub last_commit_hash: ::prost::alloc::vec::Vec<u8>,
    /// transactions
    #[prost(bytes="vec", tag="7")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[prost(bytes="vec", tag="8")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// validators for the next block
    #[prost(bytes="vec", tag="9")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus params for current block
    #[prost(bytes="vec", tag="10")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub consensus_hash: ::prost::alloc::vec::Vec<u8>,
    /// state after txs from the previous block
    #[prost(bytes="vec", tag="11")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[prost(bytes="vec", tag="12")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub last_results_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[prost(bytes="vec", tag="13")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub evidence_hash: ::prost::alloc::vec::Vec<u8>,
    /// original proposer of the block
    #[prost(bytes="vec", tag="14")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub proposer_address: ::prost::alloc::vec::Vec<u8>,
}
/// Data contains the set of transactions included in the block
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// Txs that will be applied by state @ block.Height+1.
    /// NOTE: not all txs here are valid.  We're just agreeing on the order first.
    /// This means that block.AppHash does not include these txs.
    #[prost(bytes="vec", repeated, tag="1")]
    #[serde(with = "crate::serializers::txs")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// Vote represents a prevote, precommit, or commit vote from validators for
/// consensus.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(enumeration="SignedMsgType", tag="1")]
    pub r#type: i32,
    #[prost(int64, tag="2")]
    #[serde(with = "crate::serializers::from_str")]
    pub height: i64,
    #[prost(int32, tag="3")]
    #[serde(with = "crate::serializers::from_str")]
    pub round: i32,
    /// zero if vote is nil.
    #[prost(message, optional, tag="4")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag="5")]
    #[serde(with = "crate::serializers::optional")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(bytes="vec", tag="6")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int32, tag="7")]
    #[serde(with = "crate::serializers::from_str")]
    pub validator_index: i32,
    #[prost(bytes="vec", tag="8")]
    #[serde(with = "crate::serializers::bytes::base64string")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// Commit contains the evidence that a block was committed by a set of validators.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commit {
    #[prost(int64, tag="1")]
    #[serde(with = "crate::serializers::from_str")]
    pub height: i64,
    #[prost(int32, tag="2")]
    pub round: i32,
    #[prost(message, optional, tag="3")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, repeated, tag="4")]
    #[serde(with = "crate::serializers::nullable")]
    pub signatures: ::prost::alloc::vec::Vec<CommitSig>,
}
/// CommitSig is a part of the Vote included in a Commit.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitSig {
    #[prost(enumeration="BlockIdFlag", tag="1")]
    pub block_id_flag: i32,
    #[prost(bytes="vec", tag="2")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="3")]
    #[serde(with = "crate::serializers::optional")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(bytes="vec", tag="4")]
    #[serde(with = "crate::serializers::bytes::base64string")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(enumeration="SignedMsgType", tag="1")]
    pub r#type: i32,
    #[prost(int64, tag="2")]
    pub height: i64,
    #[prost(int32, tag="3")]
    pub round: i32,
    #[prost(int32, tag="4")]
    pub pol_round: i32,
    #[prost(message, optional, tag="5")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag="6")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(bytes="vec", tag="7")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedHeader {
    #[prost(message, optional, tag="1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag="2")]
    pub commit: ::core::option::Option<Commit>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightBlock {
    #[prost(message, optional, tag="1")]
    pub signed_header: ::core::option::Option<SignedHeader>,
    #[prost(message, optional, tag="2")]
    pub validator_set: ::core::option::Option<ValidatorSet>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMeta {
    #[prost(message, optional, tag="1")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(int64, tag="2")]
    #[serde(with = "crate::serializers::from_str")]
    pub block_size: i64,
    #[prost(message, optional, tag="3")]
    pub header: ::core::option::Option<Header>,
    #[prost(int64, tag="4")]
    #[serde(with = "crate::serializers::from_str")]
    pub num_txs: i64,
}
/// TxProof represents a Merkle proof of the presence of a transaction in the Merkle tree.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxProof {
    #[prost(bytes="vec", tag="1")]
    #[serde(with = "crate::serializers::bytes::hexstring")]
    pub root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    #[serde(with = "crate::serializers::bytes::base64string")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="3")]
    pub proof: ::core::option::Option<super::crypto::Proof>,
}
/// BlockIdFlag indicates which BlcokID the signature is for
#[derive(::num_derive::FromPrimitive, ::num_derive::ToPrimitive)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BlockIdFlag {
    Unknown = 0,
    Absent = 1,
    Commit = 2,
    Nil = 3,
}
/// SignedMsgType is a type of signed message in the consensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SignedMsgType {
    Unknown = 0,
    /// Votes
    Prevote = 1,
    Precommit = 2,
    /// Proposals
    Proposal = 32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventDataRoundState {
    #[prost(int64, tag="1")]
    pub height: i64,
    #[prost(int32, tag="2")]
    pub round: i32,
    #[prost(string, tag="3")]
    pub step: ::prost::alloc::string::String,
}
/// ConsensusParams contains consensus critical parameters that determine the
/// validity of blocks.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag="1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag="2")]
    pub evidence: ::core::option::Option<EvidenceParams>,
    #[prost(message, optional, tag="3")]
    pub validator: ::core::option::Option<ValidatorParams>,
    #[prost(message, optional, tag="4")]
    pub version: ::core::option::Option<VersionParams>,
}
/// BlockParams contains limits on the block size.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Max block size, in bytes.
    /// Note: must be greater than 0
    #[prost(int64, tag="1")]
    pub max_bytes: i64,
    /// Max gas per block.
    /// Note: must be greater or equal to -1
    #[prost(int64, tag="2")]
    pub max_gas: i64,
    /// Minimum time increment between consecutive blocks (in milliseconds) If the
    /// block header timestamp is ahead of the system clock, decrease this value.
    ///
    /// Not exposed to the application.
    #[prost(int64, tag="3")]
    pub time_iota_ms: i64,
}
/// EvidenceParams determine how we handle evidence of malfeasance.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceParams {
    /// Max age of evidence, in blocks.
    ///
    /// The basic formula for calculating this is: MaxAgeDuration / {average block
    /// time}.
    #[prost(int64, tag="1")]
    pub max_age_num_blocks: i64,
    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake
    /// attacks](https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed).
    #[prost(message, optional, tag="2")]
    pub max_age_duration: ::core::option::Option<super::super::google::protobuf::Duration>,
    /// This sets the maximum size of total evidence in bytes that can be committed in a single block.
    /// and should fall comfortably under the max block bytes.
    /// Default is 1048576 or 1MB
    #[prost(int64, tag="3")]
    #[serde(with = "crate::serializers::from_str", default)]
    pub max_bytes: i64,
}
/// ValidatorParams restrict the public key types validators can use.
/// NOTE: uses ABCI pubkey naming, not Amino names.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorParams {
    #[prost(string, repeated, tag="1")]
    pub pub_key_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// VersionParams contains the ABCI application version.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionParams {
    #[prost(uint64, tag="1")]
    pub app_version: u64,
}
/// HashedParams is a subset of ConsensusParams.
///
/// It is hashed into the Header.ConsensusHash.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HashedParams {
    #[prost(int64, tag="1")]
    pub block_max_bytes: i64,
    #[prost(int64, tag="2")]
    pub block_max_gas: i64,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(from = "crate::serializers::evidence::EvidenceVariant", into = "crate::serializers::evidence::EvidenceVariant")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    #[prost(oneof="evidence::Sum", tags="1, 2")]
    pub sum: ::core::option::Option<evidence::Sum>,
}
/// Nested message and enum types in `Evidence`.
pub mod evidence {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[serde(tag = "type", content = "value")]
    #[serde(from = "crate::serializers::evidence::EvidenceVariant", into = "crate::serializers::evidence::EvidenceVariant")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        #[serde(rename = "tendermint/DuplicateVoteEvidence")]
        DuplicateVoteEvidence(super::DuplicateVoteEvidence),
        #[prost(message, tag="2")]
        #[serde(rename = "tendermint/LightClientAttackEvidence")]
        LightClientAttackEvidence(super::LightClientAttackEvidence),
    }
}
/// DuplicateVoteEvidence contains evidence of a validator signed two conflicting votes.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DuplicateVoteEvidence {
    #[prost(message, optional, tag="1")]
    pub vote_a: ::core::option::Option<Vote>,
    #[prost(message, optional, tag="2")]
    pub vote_b: ::core::option::Option<Vote>,
    #[prost(int64, tag="3")]
    pub total_voting_power: i64,
    #[prost(int64, tag="4")]
    pub validator_power: i64,
    #[prost(message, optional, tag="5")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
}
/// LightClientAttackEvidence contains evidence of a set of validators attempting to mislead a light client.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientAttackEvidence {
    #[prost(message, optional, tag="1")]
    pub conflicting_block: ::core::option::Option<LightBlock>,
    #[prost(int64, tag="2")]
    pub common_height: i64,
    #[prost(message, repeated, tag="3")]
    pub byzantine_validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(int64, tag="4")]
    pub total_voting_power: i64,
    #[prost(message, optional, tag="5")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceList {
    #[prost(message, repeated, tag="1")]
    #[serde(with = "crate::serializers::nullable")]
    pub evidence: ::prost::alloc::vec::Vec<Evidence>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalBlockId {
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    #[serde(alias = "parts")]
    pub part_set_header: ::core::option::Option<CanonicalPartSetHeader>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalPartSetHeader {
    #[prost(uint32, tag="1")]
    pub total: u32,
    #[prost(bytes="vec", tag="2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalProposal {
    /// type alias for byte
    #[prost(enumeration="SignedMsgType", tag="1")]
    pub r#type: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag="2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag="3")]
    pub round: i64,
    #[prost(int64, tag="4")]
    pub pol_round: i64,
    #[prost(message, optional, tag="5")]
    pub block_id: ::core::option::Option<CanonicalBlockId>,
    #[prost(message, optional, tag="6")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(string, tag="7")]
    pub chain_id: ::prost::alloc::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalVote {
    /// type alias for byte
    #[prost(enumeration="SignedMsgType", tag="1")]
    pub r#type: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag="2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag="3")]
    pub round: i64,
    #[prost(message, optional, tag="4")]
    pub block_id: ::core::option::Option<CanonicalBlockId>,
    #[prost(message, optional, tag="5")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(string, tag="6")]
    pub chain_id: ::prost::alloc::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, optional, tag="1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag="2")]
    pub data: ::core::option::Option<Data>,
    #[prost(message, optional, tag="3")]
    pub evidence: ::core::option::Option<EvidenceList>,
    #[prost(message, optional, tag="4")]
    pub last_commit: ::core::option::Option<Commit>,
}
