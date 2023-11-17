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
        Echo(super::super::v1beta1::RequestEcho),
        #[prost(message, tag = "2")]
        Flush(super::super::v1beta1::RequestFlush),
        #[prost(message, tag = "3")]
        Info(super::super::v1beta2::RequestInfo),
        #[prost(message, tag = "5")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag = "6")]
        Query(super::super::v1beta1::RequestQuery),
        #[prost(message, tag = "8")]
        CheckTx(super::super::v1beta1::RequestCheckTx),
        #[prost(message, tag = "11")]
        Commit(super::super::v1beta1::RequestCommit),
        #[prost(message, tag = "12")]
        ListSnapshots(super::super::v1beta1::RequestListSnapshots),
        #[prost(message, tag = "13")]
        OfferSnapshot(super::super::v1beta1::RequestOfferSnapshot),
        #[prost(message, tag = "14")]
        LoadSnapshotChunk(super::super::v1beta1::RequestLoadSnapshotChunk),
        #[prost(message, tag = "15")]
        ApplySnapshotChunk(super::super::v1beta1::RequestApplySnapshotChunk),
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
        super::super::types::v1beta3::ConsensusParams,
    >,
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<super::v1beta1::ValidatorUpdate>,
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
    pub misbehavior: ::prost::alloc::vec::Vec<super::v1beta2::Misbehavior>,
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
    pub misbehavior: ::prost::alloc::vec::Vec<super::v1beta2::Misbehavior>,
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
    /// the hash of the block that this vote may be referring to
    #[prost(bytes = "bytes", tag = "1")]
    pub hash: ::prost::bytes::Bytes,
    /// the height of the extended vote
    #[prost(int64, tag = "2")]
    pub height: i64,
    /// info of the block that this vote may be referring to
    #[prost(message, optional, tag = "3")]
    pub time: ::core::option::Option<crate::google::protobuf::Timestamp>,
    #[prost(bytes = "bytes", repeated, tag = "4")]
    pub txs: ::prost::alloc::vec::Vec<::prost::bytes::Bytes>,
    #[prost(message, optional, tag = "5")]
    pub proposed_last_commit: ::core::option::Option<CommitInfo>,
    #[prost(message, repeated, tag = "6")]
    pub misbehavior: ::prost::alloc::vec::Vec<super::v1beta2::Misbehavior>,
    #[prost(bytes = "bytes", tag = "7")]
    pub next_validators_hash: ::prost::bytes::Bytes,
    /// address of the public key of the original proposer of the block.
    #[prost(bytes = "bytes", tag = "8")]
    pub proposer_address: ::prost::bytes::Bytes,
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
    pub misbehavior: ::prost::alloc::vec::Vec<super::v1beta2::Misbehavior>,
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
        Exception(super::super::v1beta1::ResponseException),
        #[prost(message, tag = "2")]
        Echo(super::super::v1beta1::ResponseEcho),
        #[prost(message, tag = "3")]
        Flush(super::super::v1beta1::ResponseFlush),
        #[prost(message, tag = "4")]
        Info(super::super::v1beta1::ResponseInfo),
        #[prost(message, tag = "6")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag = "7")]
        Query(super::super::v1beta1::ResponseQuery),
        #[prost(message, tag = "9")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag = "12")]
        Commit(super::ResponseCommit),
        #[prost(message, tag = "13")]
        ListSnapshots(super::super::v1beta1::ResponseListSnapshots),
        #[prost(message, tag = "14")]
        OfferSnapshot(super::super::v1beta1::ResponseOfferSnapshot),
        #[prost(message, tag = "15")]
        LoadSnapshotChunk(super::super::v1beta1::ResponseLoadSnapshotChunk),
        #[prost(message, tag = "16")]
        ApplySnapshotChunk(super::super::v1beta1::ResponseApplySnapshotChunk),
        #[prost(message, tag = "17")]
        PrepareProposal(super::super::v1beta2::ResponsePrepareProposal),
        #[prost(message, tag = "18")]
        ProcessProposal(super::super::v1beta2::ResponseProcessProposal),
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
        super::super::types::v1beta3::ConsensusParams,
    >,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<super::v1beta1::ValidatorUpdate>,
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
    pub events: ::prost::alloc::vec::Vec<super::v1beta2::Event>,
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
    pub events: ::prost::alloc::vec::Vec<super::v1beta2::Event>,
    /// the result of executing each transaction including the events
    /// the particular transaction emitted. This should match the order
    /// of the transactions delivered in the block itself
    #[prost(message, repeated, tag = "2")]
    pub tx_results: ::prost::alloc::vec::Vec<ExecTxResult>,
    /// a list of updates to the validator set. These will reflect the validator set at current height + 2.
    #[prost(message, repeated, tag = "3")]
    pub validator_updates: ::prost::alloc::vec::Vec<super::v1beta1::ValidatorUpdate>,
    /// updates to the consensus params, if any.
    #[prost(message, optional, tag = "4")]
    pub consensus_param_updates: ::core::option::Option<
        super::super::types::v1beta3::ConsensusParams,
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
    pub validator: ::core::option::Option<super::v1beta1::Validator>,
    #[prost(enumeration = "super::super::types::v1beta1::BlockIdFlag", tag = "3")]
    pub block_id_flag: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedVoteInfo {
    /// The validator that sent the vote.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<super::v1beta1::Validator>,
    /// Non-deterministic extension provided by the sending validator's application.
    #[prost(bytes = "bytes", tag = "3")]
    pub vote_extension: ::prost::bytes::Bytes,
    /// Vote extension signature created by CometBFT
    #[prost(bytes = "bytes", tag = "4")]
    pub extension_signature: ::prost::bytes::Bytes,
    /// block_id_flag indicates whether the validator voted for a block, nil, or did not vote at all
    #[prost(enumeration = "super::super::types::v1beta1::BlockIdFlag", tag = "5")]
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
    pub events: ::prost::alloc::vec::Vec<super::v1beta2::Event>,
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
            request: tonic::Request<super::super::v1beta1::RequestEcho>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseEcho>,
            tonic::Status,
        >;
        async fn flush(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestFlush>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseFlush>,
            tonic::Status,
        >;
        async fn info(
            &self,
            request: tonic::Request<super::super::v1beta2::RequestInfo>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseInfo>,
            tonic::Status,
        >;
        async fn check_tx(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestCheckTx>,
        ) -> std::result::Result<tonic::Response<super::ResponseCheckTx>, tonic::Status>;
        async fn query(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestQuery>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseQuery>,
            tonic::Status,
        >;
        async fn commit(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestCommit>,
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
            request: tonic::Request<super::super::v1beta1::RequestListSnapshots>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseListSnapshots>,
            tonic::Status,
        >;
        async fn offer_snapshot(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestOfferSnapshot>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseOfferSnapshot>,
            tonic::Status,
        >;
        async fn load_snapshot_chunk(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestLoadSnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseLoadSnapshotChunk>,
            tonic::Status,
        >;
        async fn apply_snapshot_chunk(
            &self,
            request: tonic::Request<super::super::v1beta1::RequestApplySnapshotChunk>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta1::ResponseApplySnapshotChunk>,
            tonic::Status,
        >;
        async fn prepare_proposal(
            &self,
            request: tonic::Request<super::RequestPrepareProposal>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta2::ResponsePrepareProposal>,
            tonic::Status,
        >;
        async fn process_proposal(
            &self,
            request: tonic::Request<super::RequestProcessProposal>,
        ) -> std::result::Result<
            tonic::Response<super::super::v1beta2::ResponseProcessProposal>,
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
                "/cometbft.abci.v1beta3.ABCI/Echo" => {
                    #[allow(non_camel_case_types)]
                    struct EchoSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::super::v1beta1::RequestEcho>
                    for EchoSvc<T> {
                        type Response = super::super::v1beta1::ResponseEcho;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::v1beta1::RequestEcho>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::echo(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/Flush" => {
                    #[allow(non_camel_case_types)]
                    struct FlushSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::super::v1beta1::RequestFlush>
                    for FlushSvc<T> {
                        type Response = super::super::v1beta1::ResponseFlush;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::v1beta1::RequestFlush>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::flush(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/Info" => {
                    #[allow(non_camel_case_types)]
                    struct InfoSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::super::v1beta2::RequestInfo>
                    for InfoSvc<T> {
                        type Response = super::super::v1beta1::ResponseInfo;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::v1beta2::RequestInfo>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::info(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/CheckTx" => {
                    #[allow(non_camel_case_types)]
                    struct CheckTxSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::super::v1beta1::RequestCheckTx>
                    for CheckTxSvc<T> {
                        type Response = super::ResponseCheckTx;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::v1beta1::RequestCheckTx,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::check_tx(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/Query" => {
                    #[allow(non_camel_case_types)]
                    struct QuerySvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::super::v1beta1::RequestQuery>
                    for QuerySvc<T> {
                        type Response = super::super::v1beta1::ResponseQuery;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::v1beta1::RequestQuery>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::query(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/Commit" => {
                    #[allow(non_camel_case_types)]
                    struct CommitSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::super::v1beta1::RequestCommit>
                    for CommitSvc<T> {
                        type Response = super::ResponseCommit;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::v1beta1::RequestCommit>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::commit(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/InitChain" => {
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
                            let fut = async move {
                                <T as Abci>::init_chain(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/ListSnapshots" => {
                    #[allow(non_camel_case_types)]
                    struct ListSnapshotsSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<
                        super::super::v1beta1::RequestListSnapshots,
                    > for ListSnapshotsSvc<T> {
                        type Response = super::super::v1beta1::ResponseListSnapshots;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::v1beta1::RequestListSnapshots,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::list_snapshots(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/OfferSnapshot" => {
                    #[allow(non_camel_case_types)]
                    struct OfferSnapshotSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<
                        super::super::v1beta1::RequestOfferSnapshot,
                    > for OfferSnapshotSvc<T> {
                        type Response = super::super::v1beta1::ResponseOfferSnapshot;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::v1beta1::RequestOfferSnapshot,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::offer_snapshot(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/LoadSnapshotChunk" => {
                    #[allow(non_camel_case_types)]
                    struct LoadSnapshotChunkSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<
                        super::super::v1beta1::RequestLoadSnapshotChunk,
                    > for LoadSnapshotChunkSvc<T> {
                        type Response = super::super::v1beta1::ResponseLoadSnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::v1beta1::RequestLoadSnapshotChunk,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::load_snapshot_chunk(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/ApplySnapshotChunk" => {
                    #[allow(non_camel_case_types)]
                    struct ApplySnapshotChunkSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<
                        super::super::v1beta1::RequestApplySnapshotChunk,
                    > for ApplySnapshotChunkSvc<T> {
                        type Response = super::super::v1beta1::ResponseApplySnapshotChunk;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::v1beta1::RequestApplySnapshotChunk,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Abci>::apply_snapshot_chunk(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/PrepareProposal" => {
                    #[allow(non_camel_case_types)]
                    struct PrepareProposalSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestPrepareProposal>
                    for PrepareProposalSvc<T> {
                        type Response = super::super::v1beta2::ResponsePrepareProposal;
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
                                <T as Abci>::prepare_proposal(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/ProcessProposal" => {
                    #[allow(non_camel_case_types)]
                    struct ProcessProposalSvc<T: Abci>(pub Arc<T>);
                    impl<
                        T: Abci,
                    > tonic::server::UnaryService<super::RequestProcessProposal>
                    for ProcessProposalSvc<T> {
                        type Response = super::super::v1beta2::ResponseProcessProposal;
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
                                <T as Abci>::process_proposal(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/ExtendVote" => {
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
                            let fut = async move {
                                <T as Abci>::extend_vote(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/VerifyVoteExtension" => {
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
                                <T as Abci>::verify_vote_extension(&inner, request).await
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
                "/cometbft.abci.v1beta3.ABCI/FinalizeBlock" => {
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
                                <T as Abci>::finalize_block(&inner, request).await
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
        const NAME: &'static str = "cometbft.abci.v1beta3.ABCI";
    }
}
