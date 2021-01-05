// This file is copied from http://github.com/tendermint/abci
// NOTE: When using custom types, mind the warnings.
// https://github.com/gogo/protobuf/blob/master/custom_types.md#warnings-and-issues

//----------------------------------------
// Request types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(oneof="request::Value", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15")]
    pub value: ::core::option::Option<request::Value>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag="1")]
        Echo(super::RequestEcho),
        #[prost(message, tag="2")]
        Flush(super::RequestFlush),
        #[prost(message, tag="3")]
        Info(super::RequestInfo),
        #[prost(message, tag="4")]
        SetOption(super::RequestSetOption),
        #[prost(message, tag="5")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag="6")]
        Query(super::RequestQuery),
        #[prost(message, tag="7")]
        BeginBlock(super::RequestBeginBlock),
        #[prost(message, tag="8")]
        CheckTx(super::RequestCheckTx),
        #[prost(message, tag="9")]
        DeliverTx(super::RequestDeliverTx),
        #[prost(message, tag="10")]
        EndBlock(super::RequestEndBlock),
        #[prost(message, tag="11")]
        Commit(super::RequestCommit),
        #[prost(message, tag="12")]
        ListSnapshots(super::RequestListSnapshots),
        #[prost(message, tag="13")]
        OfferSnapshot(super::RequestOfferSnapshot),
        #[prost(message, tag="14")]
        LoadSnapshotChunk(super::RequestLoadSnapshotChunk),
        #[prost(message, tag="15")]
        ApplySnapshotChunk(super::RequestApplySnapshotChunk),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestEcho {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFlush {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInfo {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub block_version: u64,
    #[prost(uint64, tag="3")]
    pub p2p_version: u64,
}
/// nondeterministic
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestSetOption {
    #[prost(string, tag="1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    #[prost(message, optional, tag="1")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(string, tag="2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub consensus_params: ::core::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag="4")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes="vec", tag="5")]
    pub app_state_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="6")]
    pub initial_height: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestQuery {
    #[prost(bytes="vec", tag="1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub path: ::prost::alloc::string::String,
    #[prost(int64, tag="3")]
    pub height: i64,
    #[prost(bool, tag="4")]
    pub prove: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBeginBlock {
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    pub header: ::core::option::Option<super::types::Header>,
    #[prost(message, optional, tag="3")]
    pub last_commit_info: ::core::option::Option<LastCommitInfo>,
    #[prost(message, repeated, tag="4")]
    pub byzantine_validators: ::prost::alloc::vec::Vec<Evidence>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCheckTx {
    #[prost(bytes="vec", tag="1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration="CheckTxType", tag="2")]
    pub r#type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestDeliverTx {
    #[prost(bytes="vec", tag="1")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestEndBlock {
    #[prost(int64, tag="1")]
    pub height: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCommit {
}
/// lists available snapshots
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestListSnapshots {
}
/// offers a snapshot to the application
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestOfferSnapshot {
    /// snapshot offered by peers
    #[prost(message, optional, tag="1")]
    pub snapshot: ::core::option::Option<Snapshot>,
    /// light client-verified app hash for snapshot height
    #[prost(bytes="vec", tag="2")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
/// loads a snapshot chunk
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestLoadSnapshotChunk {
    #[prost(uint64, tag="1")]
    pub height: u64,
    #[prost(uint32, tag="2")]
    pub format: u32,
    #[prost(uint32, tag="3")]
    pub chunk: u32,
}
/// Applies a snapshot chunk
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestApplySnapshotChunk {
    #[prost(uint32, tag="1")]
    pub index: u32,
    #[prost(bytes="vec", tag="2")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="3")]
    pub sender: ::prost::alloc::string::String,
}
//----------------------------------------
// Response types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(oneof="response::Value", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16")]
    pub value: ::core::option::Option<response::Value>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag="1")]
        Exception(super::ResponseException),
        #[prost(message, tag="2")]
        Echo(super::ResponseEcho),
        #[prost(message, tag="3")]
        Flush(super::ResponseFlush),
        #[prost(message, tag="4")]
        Info(super::ResponseInfo),
        #[prost(message, tag="5")]
        SetOption(super::ResponseSetOption),
        #[prost(message, tag="6")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag="7")]
        Query(super::ResponseQuery),
        #[prost(message, tag="8")]
        BeginBlock(super::ResponseBeginBlock),
        #[prost(message, tag="9")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag="10")]
        DeliverTx(super::ResponseDeliverTx),
        #[prost(message, tag="11")]
        EndBlock(super::ResponseEndBlock),
        #[prost(message, tag="12")]
        Commit(super::ResponseCommit),
        #[prost(message, tag="13")]
        ListSnapshots(super::ResponseListSnapshots),
        #[prost(message, tag="14")]
        OfferSnapshot(super::ResponseOfferSnapshot),
        #[prost(message, tag="15")]
        LoadSnapshotChunk(super::ResponseLoadSnapshotChunk),
        #[prost(message, tag="16")]
        ApplySnapshotChunk(super::ResponseApplySnapshotChunk),
    }
}
/// nondeterministic
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseException {
    #[prost(string, tag="1")]
    pub error: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEcho {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFlush {
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInfo {
    #[prost(string, tag="1")]
    pub data: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag="3")]
    #[serde(with = "crate::serializers::from_str")]
    pub app_version: u64,
    #[prost(int64, tag="4")]
    #[serde(with = "crate::serializers::from_str")]
    pub last_block_height: i64,
    #[prost(bytes="vec", tag="5")]
    #[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
    pub last_block_app_hash: ::prost::alloc::vec::Vec<u8>,
}
/// nondeterministic
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseSetOption {
    #[prost(uint32, tag="1")]
    pub code: u32,
    /// bytes data = 2;
    #[prost(string, tag="3")]
    pub log: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub info: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag="1")]
    pub consensus_params: ::core::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag="2")]
    pub validators: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(bytes="vec", tag="3")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseQuery {
    #[prost(uint32, tag="1")]
    pub code: u32,
    /// bytes data = 2; // use "value" instead.
    ///
    /// nondeterministic
    #[prost(string, tag="3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag="4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag="5")]
    pub index: i64,
    #[prost(bytes="vec", tag="6")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="8")]
    pub proof_ops: ::core::option::Option<super::crypto::ProofOps>,
    #[prost(int64, tag="9")]
    pub height: i64,
    #[prost(string, tag="10")]
    pub codespace: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBeginBlock {
    #[prost(message, repeated, tag="1")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag="1")]
    pub code: u32,
    #[prost(bytes="vec", tag="2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag="3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag="4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag="5")]
    pub gas_wanted: i64,
    #[prost(int64, tag="6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag="7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag="8")]
    pub codespace: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseDeliverTx {
    #[prost(uint32, tag="1")]
    pub code: u32,
    #[prost(bytes="vec", tag="2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag="3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag="4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag="5")]
    pub gas_wanted: i64,
    #[prost(int64, tag="6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag="7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag="8")]
    pub codespace: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEndBlock {
    #[prost(message, repeated, tag="1")]
    pub validator_updates: ::prost::alloc::vec::Vec<ValidatorUpdate>,
    #[prost(message, optional, tag="2")]
    pub consensus_param_updates: ::core::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag="3")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCommit {
    /// reserve 1
    #[prost(bytes="vec", tag="2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub retain_height: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseListSnapshots {
    #[prost(message, repeated, tag="1")]
    pub snapshots: ::prost::alloc::vec::Vec<Snapshot>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseOfferSnapshot {
    #[prost(enumeration="response_offer_snapshot::Result", tag="1")]
    pub result: i32,
}
/// Nested message and enum types in `ResponseOfferSnapshot`.
pub mod response_offer_snapshot {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
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
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseLoadSnapshotChunk {
    #[prost(bytes="vec", tag="1")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseApplySnapshotChunk {
    #[prost(enumeration="response_apply_snapshot_chunk::Result", tag="1")]
    pub result: i32,
    /// Chunks to refetch and reapply
    #[prost(uint32, repeated, tag="2")]
    pub refetch_chunks: ::prost::alloc::vec::Vec<u32>,
    /// Chunk senders to reject and ban
    #[prost(string, repeated, tag="3")]
    pub reject_senders: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `ResponseApplySnapshotChunk`.
pub mod response_apply_snapshot_chunk {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
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
}
//----------------------------------------
// Misc.

/// ConsensusParams contains all consensus-relevant parameters
/// that can be adjusted by the abci app
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag="1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag="2")]
    pub evidence: ::core::option::Option<super::types::EvidenceParams>,
    #[prost(message, optional, tag="3")]
    pub validator: ::core::option::Option<super::types::ValidatorParams>,
    #[prost(message, optional, tag="4")]
    pub version: ::core::option::Option<super::types::VersionParams>,
}
/// BlockParams contains limits on the block size.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Note: must be greater than 0
    #[prost(int64, tag="1")]
    pub max_bytes: i64,
    /// Note: must be greater or equal to -1
    #[prost(int64, tag="2")]
    pub max_gas: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LastCommitInfo {
    #[prost(int32, tag="1")]
    pub round: i32,
    #[prost(message, repeated, tag="2")]
    pub votes: ::prost::alloc::vec::Vec<VoteInfo>,
}
/// Event allows application developers to attach additional information to
/// ResponseBeginBlock, ResponseEndBlock, ResponseCheckTx and ResponseDeliverTx.
/// Later, transactions may be queried using these events.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag="1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="2")]
    pub attributes: ::prost::alloc::vec::Vec<EventAttribute>,
}
/// EventAttribute is a single key-value pair, associated with an event.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(bytes="vec", tag="1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(bool, tag="3")]
    pub index: bool,
}
/// TxResult contains results of executing the transaction.
///
/// One usage is indexing transaction results.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxResult {
    #[prost(int64, tag="1")]
    pub height: i64,
    #[prost(uint32, tag="2")]
    pub index: u32,
    #[prost(bytes="vec", tag="3")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="4")]
    pub result: ::core::option::Option<ResponseDeliverTx>,
}
//----------------------------------------
// Blockchain Types

/// Validator
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    /// PubKey pub_key = 2 [(gogoproto.nullable)=false];
    ///
    /// The voting power
    #[prost(int64, tag="3")]
    pub power: i64,
}
/// ValidatorUpdate
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorUpdate {
    #[prost(message, optional, tag="1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag="2")]
    pub power: i64,
}
/// VoteInfo
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteInfo {
    #[prost(message, optional, tag="1")]
    pub validator: ::core::option::Option<Validator>,
    #[prost(bool, tag="2")]
    pub signed_last_block: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    #[prost(enumeration="EvidenceType", tag="1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag="2")]
    pub validator: ::core::option::Option<Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag="3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag="4")]
    pub time: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    /// https://github.com/tendermint/tendermint/issues/4581
    #[prost(int64, tag="5")]
    pub total_voting_power: i64,
}
//----------------------------------------
// State Sync Types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Snapshot {
    /// The height at which the snapshot was taken
    #[prost(uint64, tag="1")]
    pub height: u64,
    /// The application-specific snapshot format
    #[prost(uint32, tag="2")]
    pub format: u32,
    /// Number of chunks in the snapshot
    #[prost(uint32, tag="3")]
    pub chunks: u32,
    /// Arbitrary snapshot hash, equal only if identical
    #[prost(bytes="vec", tag="4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// Arbitrary application metadata
    #[prost(bytes="vec", tag="5")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CheckTxType {
    New = 0,
    Recheck = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EvidenceType {
    Unknown = 0,
    DuplicateVote = 1,
    LightClientAttack = 2,
}
