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
        Echo(super::RequestEcho),
        #[prost(message, tag = "2")]
        Flush(super::RequestFlush),
        #[prost(message, tag = "3")]
        Info(super::RequestInfo),
        #[prost(message, tag = "5")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag = "6")]
        Query(super::RequestQuery),
        #[prost(message, tag = "8")]
        CheckTx(super::RequestCheckTx),
        #[prost(message, tag = "11")]
        Commit(super::RequestCommit),
        #[prost(message, tag = "12")]
        ListSnapshots(super::RequestListSnapshots),
        #[prost(message, tag = "13")]
        OfferSnapshot(super::RequestOfferSnapshot),
        #[prost(message, tag = "14")]
        LoadSnapshotChunk(super::RequestLoadSnapshotChunk),
        #[prost(message, tag = "15")]
        ApplySnapshotChunk(super::RequestApplySnapshotChunk),
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
pub struct RequestEcho {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFlush {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInfo {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub block_version: u64,
    #[prost(uint64, tag = "3")]
    pub p2p_version: u64,
    #[prost(string, tag = "4")]
    pub abci_version: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes = "bytes", tag = "5")]
    pub app_state_bytes: ::prost::bytes::Bytes,
    #[prost(int64, tag = "6")]
    pub initial_height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestQuery {
    #[prost(bytes = "bytes", tag = "1")]
    pub data: ::prost::bytes::Bytes,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bool, tag = "4")]
    pub prove: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCheckTx {
    #[prost(bytes = "bytes", tag = "1")]
    pub tx: ::prost::bytes::Bytes,
    #[prost(enumeration = "CheckTxType", tag = "2")]
    pub r#type: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCommit {}
/// lists available snapshots
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestListSnapshots {}
/// offers a snapshot to the application
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestOfferSnapshot {
    /// snapshot offered by peers
    #[prost(message, optional, tag = "1")]
    pub snapshot: ::core::option::Option<Snapshot>,
    /// light client-verified app hash for snapshot height
    #[prost(bytes = "bytes", tag = "2")]
    pub app_hash: ::prost::bytes::Bytes,
}
/// loads a snapshot chunk
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestLoadSnapshotChunk {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(uint32, tag = "2")]
    pub format: u32,
    #[prost(uint32, tag = "3")]
    pub chunk: u32,
}
/// Applies a snapshot chunk
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestApplySnapshotChunk {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "bytes", tag = "2")]
    pub chunk: ::prost::bytes::Bytes,
    #[prost(string, tag = "3")]
    pub sender: ::prost::alloc::string::String,
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
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
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
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
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
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
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
        Exception(super::ResponseException),
        #[prost(message, tag = "2")]
        Echo(super::ResponseEcho),
        #[prost(message, tag = "3")]
        Flush(super::ResponseFlush),
        #[prost(message, tag = "4")]
        Info(super::ResponseInfo),
        #[prost(message, tag = "6")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag = "7")]
        Query(super::ResponseQuery),
        #[prost(message, tag = "9")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag = "12")]
        Commit(super::ResponseCommit),
        #[prost(message, tag = "13")]
        ListSnapshots(super::ResponseListSnapshots),
        #[prost(message, tag = "14")]
        OfferSnapshot(super::ResponseOfferSnapshot),
        #[prost(message, tag = "15")]
        LoadSnapshotChunk(super::ResponseLoadSnapshotChunk),
        #[prost(message, tag = "16")]
        ApplySnapshotChunk(super::ResponseApplySnapshotChunk),
        #[prost(message, tag = "17")]
        PrepareProposal(super::ResponsePrepareProposal),
        #[prost(message, tag = "18")]
        ProcessProposal(super::ResponseProcessProposal),
        #[prost(message, tag = "19")]
        ExtendVote(super::ResponseExtendVote),
        #[prost(message, tag = "20")]
        VerifyVoteExtension(super::ResponseVerifyVoteExtension),
        #[prost(message, tag = "21")]
        FinalizeBlock(super::ResponseFinalizeBlock),
    }
}
/// nondeterministic
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseException {
    #[prost(string, tag = "1")]
    pub error: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEcho {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFlush {}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInfo {
    #[prost(string, tag = "1")]
    #[serde(default)]
    pub data: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    #[serde(default)]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(with = "crate::serializers::from_str", default)]
    pub app_version: u64,
    #[prost(int64, tag = "4")]
    #[serde(with = "crate::serializers::from_str", default)]
    pub last_block_height: i64,
    #[prost(bytes = "bytes", tag = "5")]
    #[serde(default)]
    #[serde(skip_serializing_if = "bytes::Bytes::is_empty")]
    pub last_block_app_hash: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag = "1")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes = "bytes", tag = "3")]
    pub app_hash: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseQuery {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// bytes data = 2; // use "value" instead.
    ///
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub index: i64,
    #[prost(bytes = "bytes", tag = "6")]
    pub key: ::prost::bytes::Bytes,
    #[prost(bytes = "bytes", tag = "7")]
    pub value: ::prost::bytes::Bytes,
    #[prost(message, optional, tag = "8")]
    pub proof_ops: ::core::option::Option<super::crypto::ProofOps>,
    #[prost(int64, tag = "9")]
    pub height: i64,
    #[prost(string, tag = "10")]
    pub codespace: ::prost::alloc::string::String,
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
    pub events: ::prost::alloc::vec::Vec<Event>,
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
pub struct ResponseListSnapshots {
    #[prost(message, repeated, tag = "1")]
    pub snapshots: ::prost::alloc::vec::Vec<Snapshot>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseOfferSnapshot {
    #[prost(enumeration = "response_offer_snapshot::Result", tag = "1")]
    pub result: i32,
}
/// Nested message and enum types in `ResponseOfferSnapshot`.
pub mod response_offer_snapshot {
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
    pub enum Result {
        /// Unknown result, abort all snapshot restoration
        Unknown = 0,
        /// Snapshot accepted, apply chunks
        Accept = 1,
        /// Abort all snapshot restoration
        Abort = 2,
        /// Reject this specific snapshot, try others
        Reject = 3,
        /// Reject all snapshots of this format, try others
        RejectFormat = 4,
        /// Reject all snapshots from the sender(s), try others
        RejectSender = 5,
    }
    impl Result {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Result::Unknown => "UNKNOWN",
                Result::Accept => "ACCEPT",
                Result::Abort => "ABORT",
                Result::Reject => "REJECT",
                Result::RejectFormat => "REJECT_FORMAT",
                Result::RejectSender => "REJECT_SENDER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "ABORT" => Some(Self::Abort),
                "REJECT" => Some(Self::Reject),
                "REJECT_FORMAT" => Some(Self::RejectFormat),
                "REJECT_SENDER" => Some(Self::RejectSender),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseLoadSnapshotChunk {
    #[prost(bytes = "bytes", tag = "1")]
    pub chunk: ::prost::bytes::Bytes,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseApplySnapshotChunk {
    #[prost(enumeration = "response_apply_snapshot_chunk::Result", tag = "1")]
    pub result: i32,
    /// Chunks to refetch and reapply
    #[prost(uint32, repeated, tag = "2")]
    pub refetch_chunks: ::prost::alloc::vec::Vec<u32>,
    /// Chunk senders to reject and ban
    #[prost(string, repeated, tag = "3")]
    pub reject_senders: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `ResponseApplySnapshotChunk`.
pub mod response_apply_snapshot_chunk {
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
    pub enum Result {
        /// Unknown result, abort all snapshot restoration
        Unknown = 0,
        /// Chunk successfully accepted
        Accept = 1,
        /// Abort all snapshot restoration
        Abort = 2,
        /// Retry chunk (combine with refetch and reject)
        Retry = 3,
        /// Retry snapshot (combine with refetch and reject)
        RetrySnapshot = 4,
        /// Reject this snapshot, try others
        RejectSnapshot = 5,
    }
    impl Result {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Result::Unknown => "UNKNOWN",
                Result::Accept => "ACCEPT",
                Result::Abort => "ABORT",
                Result::Retry => "RETRY",
                Result::RetrySnapshot => "RETRY_SNAPSHOT",
                Result::RejectSnapshot => "REJECT_SNAPSHOT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "ABORT" => Some(Self::Abort),
                "RETRY" => Some(Self::Retry),
                "RETRY_SNAPSHOT" => Some(Self::RetrySnapshot),
                "REJECT_SNAPSHOT" => Some(Self::RejectSnapshot),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponsePrepareProposal {
    #[prost(bytes = "bytes", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::bytes::Bytes>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseProcessProposal {
    #[prost(enumeration = "response_process_proposal::ProposalStatus", tag = "1")]
    pub status: i32,
}
/// Nested message and enum types in `ResponseProcessProposal`.
pub mod response_process_proposal {
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
    pub enum ProposalStatus {
        Unknown = 0,
        Accept = 1,
        Reject = 2,
    }
    impl ProposalStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ProposalStatus::Unknown => "UNKNOWN",
                ProposalStatus::Accept => "ACCEPT",
                ProposalStatus::Reject => "REJECT",
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
    pub events: ::prost::alloc::vec::Vec<Event>,
    /// the result of executing each transaction including the events
    /// the particular transction emitted. This should match the order
    /// of the transactions delivered in the block itself
    #[prost(message, repeated, tag = "2")]
    pub tx_results: ::prost::alloc::vec::Vec<ExecTxResult>,
    /// a list of updates to the validator set. These will reflect the validator set at current height + 2.
    #[prost(message, repeated, tag = "3")]
    pub validator_updates: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    /// updates to the consensus params, if any.
    #[prost(message, optional, tag = "4")]
    pub consensus_param_updates: ::core::option::Option<super::types::ConsensusParams>,
    /// app_hash is the hash of the applications' state which is used to confirm that execution of the transactions was deterministic. It is up to the application to decide which algorithm to use.
    #[prost(bytes = "bytes", tag = "5")]
    pub app_hash: ::prost::bytes::Bytes,
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
/// the PrepareProposal request such that CometBFT can provide vote extensions
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
/// Event allows application developers to attach additional information to
/// ResponseFinalizeBlock and ResponseCheckTx.
/// Later, transactions may be queried using these events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: ::prost::alloc::vec::Vec<EventAttribute>,
}
/// EventAttribute is a single key-value pair, associated with an event.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(bool, tag = "3")]
    pub index: bool,
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
    pub events: ::prost::alloc::vec::Vec<Event>,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    #[prost(bytes = "bytes", tag = "1")]
    pub address: ::prost::bytes::Bytes,
    /// PubKey pub_key = 2 \[(gogoproto.nullable)=false\];
    ///
    /// The voting power
    #[prost(int64, tag = "3")]
    pub power: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorUpdate {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag = "2")]
    pub power: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteInfo {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    #[prost(enumeration = "super::types::BlockIdFlag", tag = "3")]
    pub block_id_flag: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedVoteInfo {
    /// The validator that sent the vote.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    /// Non-deterministic extension provided by the sending validator's application.
    #[prost(bytes = "bytes", tag = "3")]
    pub vote_extension: ::prost::bytes::Bytes,
    /// Vote extension signature created by CometBFT
    #[prost(bytes = "bytes", tag = "4")]
    pub extension_signature: ::prost::bytes::Bytes,
    /// block_id_flag indicates whether the validator voted for a block, nil, or did not vote at all
    #[prost(enumeration = "super::types::BlockIdFlag", tag = "5")]
    pub block_id_flag: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehavior {
    #[prost(enumeration = "MisbehaviorType", tag = "1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag = "2")]
    pub validator: ::core::option::Option<Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    /// <https://github.com/tendermint/tendermint/issues/4581>
    #[prost(int64, tag = "5")]
    pub total_voting_power: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Snapshot {
    /// The height at which the snapshot was taken
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// The application-specific snapshot format
    #[prost(uint32, tag = "2")]
    pub format: u32,
    /// Number of chunks in the snapshot
    #[prost(uint32, tag = "3")]
    pub chunks: u32,
    /// Arbitrary snapshot hash, equal only if identical
    #[prost(bytes = "bytes", tag = "4")]
    pub hash: ::prost::bytes::Bytes,
    /// Arbitrary application metadata
    #[prost(bytes = "bytes", tag = "5")]
    pub metadata: ::prost::bytes::Bytes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CheckTxType {
    New = 0,
    Recheck = 1,
}
impl CheckTxType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CheckTxType::New => "NEW",
            CheckTxType::Recheck => "RECHECK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NEW" => Some(Self::New),
            "RECHECK" => Some(Self::Recheck),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MisbehaviorType {
    Unknown = 0,
    DuplicateVote = 1,
    LightClientAttack = 2,
}
impl MisbehaviorType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MisbehaviorType::Unknown => "UNKNOWN",
            MisbehaviorType::DuplicateVote => "DUPLICATE_VOTE",
            MisbehaviorType::LightClientAttack => "LIGHT_CLIENT_ATTACK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "DUPLICATE_VOTE" => Some(Self::DuplicateVote),
            "LIGHT_CLIENT_ATTACK" => Some(Self::LightClientAttack),
            _ => None,
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "grpc-server")]
pub mod abci_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AbciServer.
    #[async_trait]
    pub trait Abci: Send + Sync + 'static {
        async fn echo(
            &self,
            request: tonic::Request<super::RequestEcho>,
        ) -> std::result::Result<tonic::Response<super::ResponseEcho>, tonic::Status>;
        async fn flush(
            &self,
            request: tonic::Request<super::RequestFlush>,
        ) -> std::result::Result<tonic::Response<super::ResponseFlush>, tonic::Status>;
        async fn info(
            &self,
            request: tonic::Request<super::RequestInfo>,
        ) -> std::result::Result<tonic::Response<super::ResponseInfo>, tonic::Status>;
        async fn check_tx(
            &self,
            request: tonic::Request<super::RequestCheckTx>,
        ) -> std::result::Result<tonic::Response<super::ResponseCheckTx>, tonic::Status>;
        async fn query(
            &self,
            request: tonic::Request<super::RequestQuery>,
        ) -> std::result::Result<tonic::Response<super::ResponseQuery>, tonic::Status>;
        async fn commit(
            &self,
            request: tonic::Request<super::RequestCommit>,
        ) -> std::result::Result<tonic::Response<super::ResponseCommit>, tonic::Status>;
        async fn init_chain(
            &self,
            request: tonic::Request<super::RequestInitChain>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseInitChain>,
            tonic::Status,
        >;
        async fn list_snapshots(
            &self,
            request: tonic::Request<super::RequestListSnapshots>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseListSnapshots>,
            tonic::Status,
        >;
        async fn offer_snapshot(
            &self,
            request: tonic::Request<super::RequestOfferSnapshot>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseOfferSnapshot>,
            tonic::Status,
        >;
        async fn load_snapshot_chunk(
            &self,
            request: tonic::Request<super::RequestLoadSnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseLoadSnapshotChunk>,
            tonic::Status,
        >;
        async fn apply_snapshot_chunk(
            &self,
            request: tonic::Request<super::RequestApplySnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseApplySnapshotChunk>,
            tonic::Status,
        >;
        async fn prepare_proposal(
            &self,
            request: tonic::Request<super::RequestPrepareProposal>,
        ) -> std::result::Result<
            tonic::Response<super::ResponsePrepareProposal>,
            tonic::Status,
        >;
        async fn process_proposal(
            &self,
            request: tonic::Request<super::RequestProcessProposal>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseProcessProposal>,
            tonic::Status,
        >;
        async fn extend_vote(
            &self,
            request: tonic::Request<super::RequestExtendVote>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseExtendVote>,
            tonic::Status,
        >;
        async fn verify_vote_extension(
            &self,
            request: tonic::Request<super::RequestVerifyVoteExtension>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseVerifyVoteExtension>,
            tonic::Status,
        >;
        async fn finalize_block(
            &self,
            request: tonic::Request<super::RequestFinalizeBlock>,
        ) -> std::result::Result<
            tonic::Response<super::ResponseFinalizeBlock>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct AbciServer<T: Abci> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Abci> AbciServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AbciServer<T>
    where
        T: Abci,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/tendermint.abci.ABCI/Echo" => {
                    #[allow(non_camel_case_types)]
                    struct EchoSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestEcho>
                    for EchoSvc<T> {
                        type Response = super::ResponseEcho;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestEcho>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).echo(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EchoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/Flush" => {
                    #[allow(non_camel_case_types)]
                    struct FlushSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestFlush>
                    for FlushSvc<T> {
                        type Response = super::ResponseFlush;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestFlush>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).flush(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FlushSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/Info" => {
                    #[allow(non_camel_case_types)]
                    struct InfoSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestInfo>
                    for InfoSvc<T> {
                        type Response = super::ResponseInfo;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestInfo>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/CheckTx" => {
                    #[allow(non_camel_case_types)]
                    struct CheckTxSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestCheckTx>
                    for CheckTxSvc<T> {
                        type Response = super::ResponseCheckTx;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestCheckTx>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).check_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CheckTxSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/Query" => {
                    #[allow(non_camel_case_types)]
                    struct QuerySvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestQuery>
                    for QuerySvc<T> {
                        type Response = super::ResponseQuery;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestQuery>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).query(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = QuerySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/Commit" => {
                    #[allow(non_camel_case_types)]
                    struct CommitSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestCommit>
                    for CommitSvc<T> {
                        type Response = super::ResponseCommit;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestCommit>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).commit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommitSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/InitChain" => {
                    #[allow(non_camel_case_types)]
                    struct InitChainSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestInitChain>
                    for InitChainSvc<T> {
                        type Response = super::ResponseInitChain;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestInitChain>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).init_chain(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InitChainSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/ListSnapshots" => {
                    #[allow(non_camel_case_types)]
                    struct ListSnapshotsSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestListSnapshots>
                    for ListSnapshotsSvc<T> {
                        type Response = super::ResponseListSnapshots;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestListSnapshots>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).list_snapshots(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListSnapshotsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/OfferSnapshot" => {
                    #[allow(non_camel_case_types)]
                    struct OfferSnapshotSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestOfferSnapshot>
                    for OfferSnapshotSvc<T> {
                        type Response = super::ResponseOfferSnapshot;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestOfferSnapshot>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).offer_snapshot(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OfferSnapshotSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/LoadSnapshotChunk" => {
                    #[allow(non_camel_case_types)]
                    struct LoadSnapshotChunkSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestLoadSnapshotChunk>
                    for LoadSnapshotChunkSvc<T> {
                        type Response = super::ResponseLoadSnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestLoadSnapshotChunk>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).load_snapshot_chunk(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LoadSnapshotChunkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/ApplySnapshotChunk" => {
                    #[allow(non_camel_case_types)]
                    struct ApplySnapshotChunkSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestApplySnapshotChunk>
                    for ApplySnapshotChunkSvc<T> {
                        type Response = super::ResponseApplySnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestApplySnapshotChunk>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).apply_snapshot_chunk(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ApplySnapshotChunkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/PrepareProposal" => {
                    #[allow(non_camel_case_types)]
                    struct PrepareProposalSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestPrepareProposal>
                    for PrepareProposalSvc<T> {
                        type Response = super::ResponsePrepareProposal;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestPrepareProposal>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).prepare_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PrepareProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/ProcessProposal" => {
                    #[allow(non_camel_case_types)]
                    struct ProcessProposalSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestProcessProposal>
                    for ProcessProposalSvc<T> {
                        type Response = super::ResponseProcessProposal;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestProcessProposal>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).process_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProcessProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/ExtendVote" => {
                    #[allow(non_camel_case_types)]
                    struct ExtendVoteSvc<T: Abci>(pub Arc<T>);
                    impl<T: Abci> tonic::server::UnaryService<super::RequestExtendVote>
                    for ExtendVoteSvc<T> {
                        type Response = super::ResponseExtendVote;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestExtendVote>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).extend_vote(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ExtendVoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/VerifyVoteExtension" => {
                    #[allow(non_camel_case_types)]
                    struct VerifyVoteExtensionSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestVerifyVoteExtension>
                    for VerifyVoteExtensionSvc<T> {
                        type Response = super::ResponseVerifyVoteExtension;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestVerifyVoteExtension>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).verify_vote_extension(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VerifyVoteExtensionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.abci.ABCI/FinalizeBlock" => {
                    #[allow(non_camel_case_types)]
                    struct FinalizeBlockSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestFinalizeBlock>
                    for FinalizeBlockSvc<T> {
                        type Response = super::ResponseFinalizeBlock;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RequestFinalizeBlock>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).finalize_block(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FinalizeBlockSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Abci> Clone for AbciServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Abci> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Abci> tonic::server::NamedService for AbciServer<T> {
        const NAME: &'static str = "tendermint.abci.ABCI";
    }
}
