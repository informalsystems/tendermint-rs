#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(
        oneof = "request::Value",
        tags = "1, 2, 3, 5, 6, 8, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20"
    )]
    pub value: ::core::option::Option<request::Value>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Echo(super::super::v1::RequestEcho),
        #[prost(message, tag = "2")]
        Flush(super::super::v1::RequestFlush),
        #[prost(message, tag = "3")]
        Info(super::super::v2::RequestInfo),
        #[prost(message, tag = "5")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag = "6")]
        Query(super::super::v1::RequestQuery),
        #[prost(message, tag = "8")]
        CheckTx(super::super::v1::RequestCheckTx),
        #[prost(message, tag = "11")]
        Commit(super::super::v1::RequestCommit),
        #[prost(message, tag = "12")]
        ListSnapshots(super::super::v1::RequestListSnapshots),
        #[prost(message, tag = "13")]
        OfferSnapshot(super::super::v1::RequestOfferSnapshot),
        #[prost(message, tag = "14")]
        LoadSnapshotChunk(super::super::v1::RequestLoadSnapshotChunk),
        #[prost(message, tag = "15")]
        ApplySnapshotChunk(super::super::v1::RequestApplySnapshotChunk),
        #[prost(message, tag = "16")]
        PrepareProposal(super::RequestPrepareProposal),
        #[prost(message, tag = "17")]
        ProcessProposal(super::RequestProcessProposal),
        #[prost(message, tag = "18")]
        ExtendVote(super::RequestExtendVote),
        #[prost(message, tag = "19")]
        VerifyVoteExtension(super::RequestVerifyVoteExtension),
        #[prost(message, tag = "20")]
        FinalizeBlock(super::RequestFinalizeBlock),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub consensus_params: ::core::option::Option<
        super::super::types::v3::ConsensusParams,
    >,
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<super::v1::ValidatorUpdate>,
    #[prost(bytes = "bytes", tag = "5")]
    pub app_state_bytes: ::prost::bytes::Bytes,
    #[prost(int64, tag = "6")]
    pub initial_height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestPrepareProposal {
    /// the modified transactions cannot exceed this size.
    #[prost(int64, tag = "1")]
    pub max_tx_bytes: i64,
    /// txs is an array of transactions that will be included in a block,
    /// sent to the app for possible modifications.
    #[prost(bytes = "bytes", repeated, tag = "2")]
    pub txs: ::prost::alloc::vec::Vec<::prost::bytes::Bytes>,
    #[prost(message, optional, tag = "3")]
    pub local_last_commit: ::core::option::Option<ExtendedCommitInfo>,
    #[prost(message, repeated, tag = "4")]
    pub misbehavior: ::prost::alloc::vec::Vec<super::v2::Misbehavior>,
    #[prost(int64, tag = "5")]
    pub height: i64,
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(bytes = "bytes", tag = "7")]
    pub next_validators_hash: ::prost::bytes::Bytes,
    /// address of the public key of the validator proposing the block.
    #[prost(bytes = "bytes", tag = "8")]
    pub proposer_address: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestProcessProposal {
    #[prost(bytes = "bytes", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::bytes::Bytes>,
    #[prost(message, optional, tag = "2")]
    pub proposed_last_commit: ::core::option::Option<CommitInfo>,
    #[prost(message, repeated, tag = "3")]
    pub misbehavior: ::prost::alloc::vec::Vec<super::v2::Misbehavior>,
    /// hash is the merkle root hash of the fields of the proposed block.
    #[prost(bytes = "bytes", tag = "4")]
    pub hash: ::prost::bytes::Bytes,
    #[prost(int64, tag = "5")]
    pub height: i64,
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(bytes = "bytes", tag = "7")]
    pub next_validators_hash: ::prost::bytes::Bytes,
    /// address of the public key of the original proposer of the block.
    #[prost(bytes = "bytes", tag = "8")]
    pub proposer_address: ::prost::bytes::Bytes,
}
/// Extends a vote with application-injected data
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestExtendVote {
    /// the hash of the block  that this vote may be referring to
    #[prost(bytes = "bytes", tag = "1")]
    pub hash: ::prost::bytes::Bytes,
    /// the height of the extended vote
    #[prost(int64, tag = "2")]
    pub height: i64,
}
/// Verify the vote extension
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestVerifyVoteExtension {
    /// the hash of the block that this received vote corresponds to
    #[prost(bytes = "bytes", tag = "1")]
    pub hash: ::prost::bytes::Bytes,
    /// the validator that signed the vote extension
    #[prost(bytes = "bytes", tag = "2")]
    pub validator_address: ::prost::bytes::Bytes,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bytes = "bytes", tag = "4")]
    pub vote_extension: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFinalizeBlock {
    #[prost(bytes = "bytes", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::bytes::Bytes>,
    #[prost(message, optional, tag = "2")]
    pub decided_last_commit: ::core::option::Option<CommitInfo>,
    #[prost(message, repeated, tag = "3")]
    pub misbehavior: ::prost::alloc::vec::Vec<super::v2::Misbehavior>,
    /// hash is the merkle root hash of the fields of the decided block.
    #[prost(bytes = "bytes", tag = "4")]
    pub hash: ::prost::bytes::Bytes,
    #[prost(int64, tag = "5")]
    pub height: i64,
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(bytes = "bytes", tag = "7")]
    pub next_validators_hash: ::prost::bytes::Bytes,
    /// proposer_address is the address of the public key of the original proposer of the block.
    #[prost(bytes = "bytes", tag = "8")]
    pub proposer_address: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(
        oneof = "response::Value",
        tags = "1, 2, 3, 4, 6, 7, 9, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21"
    )]
    pub value: ::core::option::Option<response::Value>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Exception(super::super::v1::ResponseException),
        #[prost(message, tag = "2")]
        Echo(super::super::v1::ResponseEcho),
        #[prost(message, tag = "3")]
        Flush(super::super::v1::ResponseFlush),
        #[prost(message, tag = "4")]
        Info(super::super::v1::ResponseInfo),
        #[prost(message, tag = "6")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag = "7")]
        Query(super::super::v1::ResponseQuery),
        #[prost(message, tag = "9")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag = "12")]
        Commit(super::ResponseCommit),
        #[prost(message, tag = "13")]
        ListSnapshots(super::super::v1::ResponseListSnapshots),
        #[prost(message, tag = "14")]
        OfferSnapshot(super::super::v1::ResponseOfferSnapshot),
        #[prost(message, tag = "15")]
        LoadSnapshotChunk(super::super::v1::ResponseLoadSnapshotChunk),
        #[prost(message, tag = "16")]
        ApplySnapshotChunk(super::super::v1::ResponseApplySnapshotChunk),
        #[prost(message, tag = "17")]
        PrepareProposal(super::super::v2::ResponsePrepareProposal),
        #[prost(message, tag = "18")]
        ProcessProposal(super::super::v2::ResponseProcessProposal),
        #[prost(message, tag = "19")]
        ExtendVote(super::ResponseExtendVote),
        #[prost(message, tag = "20")]
        VerifyVoteExtension(super::ResponseVerifyVoteExtension),
        #[prost(message, tag = "21")]
        FinalizeBlock(super::ResponseFinalizeBlock),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag = "1")]
    pub consensus_params: ::core::option::Option<
        super::super::types::v3::ConsensusParams,
    >,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<super::v1::ValidatorUpdate>,
    #[prost(bytes = "bytes", tag = "3")]
    pub app_hash: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "bytes", tag = "2")]
    pub data: ::prost::bytes::Bytes,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<super::v2::Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCommit {
    #[prost(int64, tag = "3")]
    pub retain_height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseExtendVote {
    #[prost(bytes = "bytes", tag = "1")]
    pub vote_extension: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseVerifyVoteExtension {
    #[prost(enumeration = "response_verify_vote_extension::VerifyStatus", tag = "1")]
    pub status: i32,
}
/// Nested message and enum types in `ResponseVerifyVoteExtension`.
pub mod response_verify_vote_extension {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum VerifyStatus {
        Unknown = 0,
        Accept = 1,
        /// Rejecting the vote extension will reject the entire precommit by the sender.
        /// Incorrectly implementing this thus has liveness implications as it may affect
        /// CometBFT's ability to receive 2/3+ valid votes to finalize the block.
        /// Honest nodes should never be rejected.
        Reject = 2,
    }
    impl VerifyStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                VerifyStatus::Unknown => "UNKNOWN",
                VerifyStatus::Accept => "ACCEPT",
                VerifyStatus::Reject => "REJECT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "REJECT" => Some(Self::Reject),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFinalizeBlock {
    /// set of block events emmitted as part of executing the block
    #[prost(message, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<super::v2::Event>,
    /// the result of executing each transaction including the events
    /// the particular transction emitted. This should match the order
    /// of the transactions delivered in the block itself
    #[prost(message, repeated, tag = "2")]
    pub tx_results: ::prost::alloc::vec::Vec<ExecTxResult>,
    /// a list of updates to the validator set. These will reflect the validator set at current height + 2.
    #[prost(message, repeated, tag = "3")]
    pub validator_updates: ::prost::alloc::vec::Vec<super::v1::ValidatorUpdate>,
    /// updates to the consensus params, if any.
    #[prost(message, optional, tag = "4")]
    pub consensus_param_updates: ::core::option::Option<
        super::super::types::v3::ConsensusParams,
    >,
    /// app_hash is the hash of the applications' state which is used to confirm
    /// that execution of the transactions was deterministic.
    /// It is up to the application to decide which algorithm to use.
    #[prost(bytes = "bytes", tag = "5")]
    pub app_hash: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteInfo {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<super::v1::Validator>,
    #[prost(enumeration = "super::super::types::v1::BlockIdFlag", tag = "3")]
    pub block_id_flag: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedVoteInfo {
    /// The validator that sent the vote.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<super::v1::Validator>,
    /// Non-deterministic extension provided by the sending validator's application.
    #[prost(bytes = "bytes", tag = "3")]
    pub vote_extension: ::prost::bytes::Bytes,
    /// Vote extension signature created by CometBFT
    #[prost(bytes = "bytes", tag = "4")]
    pub extension_signature: ::prost::bytes::Bytes,
    /// block_id_flag indicates whether the validator voted for a block, nil, or did not vote at all
    #[prost(enumeration = "super::super::types::v1::BlockIdFlag", tag = "5")]
    pub block_id_flag: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitInfo {
    #[prost(int32, tag = "1")]
    pub round: i32,
    #[prost(message, repeated, tag = "2")]
    pub votes: ::prost::alloc::vec::Vec<VoteInfo>,
}
/// ExtendedCommitInfo is similar to CommitInfo except that it is only used in
/// the PrepareProposal request such that Tendermint can provide vote extensions
/// to the application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedCommitInfo {
    /// The round at which the block proposer decided in the previous height.
    #[prost(int32, tag = "1")]
    pub round: i32,
    /// List of validators' addresses in the last validator set with their voting
    /// information, including vote extensions.
    #[prost(message, repeated, tag = "2")]
    pub votes: ::prost::alloc::vec::Vec<ExtendedVoteInfo>,
}
/// ExecTxResult contains results of executing one individual transaction.
///
/// * Its structure is equivalent to #ResponseDeliverTx which will be deprecated/deleted
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecTxResult {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "bytes", tag = "2")]
    pub data: ::prost::bytes::Bytes,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    /// nondeterministic
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<super::v2::Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
}
/// TxResult contains results of executing the transaction.
///
/// One usage is indexing transaction results.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxResult {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(uint32, tag = "2")]
    pub index: u32,
    #[prost(bytes = "bytes", tag = "3")]
    pub tx: ::prost::bytes::Bytes,
    #[prost(message, optional, tag = "4")]
    pub result: ::core::option::Option<ExecTxResult>,
}
