// This file is copied from http://github.com/tendermint/abci
// NOTE: When using custom types, mind the warnings.
// https://github.com/gogo/protobuf/blob/master/custom_types.md#warnings-and-issues

//----------------------------------------
// Request types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(oneof="request::Value", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15")]
    pub value: ::std::option::Option<request::Value>,
}
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
    pub message: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFlush {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInfo {
    #[prost(string, tag="1")]
    pub version: std::string::String,
    #[prost(uint64, tag="2")]
    pub block_version: u64,
    #[prost(uint64, tag="3")]
    pub p2p_version: u64,
}
/// nondeterministic
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestSetOption {
    #[prost(string, tag="1")]
    pub key: std::string::String,
    #[prost(string, tag="2")]
    pub value: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    #[prost(message, optional, tag="1")]
    pub time: ::std::option::Option<super::super::google::protobuf::Timestamp>,
    #[prost(string, tag="2")]
    pub chain_id: std::string::String,
    #[prost(message, optional, tag="3")]
    pub consensus_params: ::std::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag="4")]
    pub validators: ::std::vec::Vec<ValidatorUpdate>,
    #[prost(bytes, tag="5")]
    pub app_state_bytes: std::vec::Vec<u8>,
    #[prost(int64, tag="6")]
    pub initial_height: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestQuery {
    #[prost(bytes, tag="1")]
    pub data: std::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub path: std::string::String,
    #[prost(int64, tag="3")]
    pub height: i64,
    #[prost(bool, tag="4")]
    pub prove: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBeginBlock {
    #[prost(bytes, tag="1")]
    pub hash: std::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    pub header: ::std::option::Option<super::types::Header>,
    #[prost(message, optional, tag="3")]
    pub last_commit_info: ::std::option::Option<LastCommitInfo>,
    #[prost(message, repeated, tag="4")]
    pub byzantine_validators: ::std::vec::Vec<Evidence>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestCheckTx {
    #[prost(bytes, tag="1")]
    pub tx: std::vec::Vec<u8>,
    #[prost(enumeration="CheckTxType", tag="2")]
    pub r#type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestDeliverTx {
    #[prost(bytes, tag="1")]
    pub tx: std::vec::Vec<u8>,
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
    pub snapshot: ::std::option::Option<Snapshot>,
    /// light client-verified app hash for snapshot height
    #[prost(bytes, tag="2")]
    pub app_hash: std::vec::Vec<u8>,
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
    #[prost(bytes, tag="2")]
    pub chunk: std::vec::Vec<u8>,
    #[prost(string, tag="3")]
    pub sender: std::string::String,
}
//----------------------------------------
// Response types

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(oneof="response::Value", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16")]
    pub value: ::std::option::Option<response::Value>,
}
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
    pub error: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEcho {
    #[prost(string, tag="1")]
    pub message: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFlush {
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct ResponseInfo {
    #[prost(string, tag="1")]
    pub data: std::string::String,
    #[prost(string, tag="2")]
    pub version: std::string::String,
    #[prost(uint64, tag="3")]
    #[serde(with = "crate::serializers::from_str")]
    pub app_version: u64,
    #[prost(int64, tag="4")]
    #[serde(with = "crate::serializers::from_str")]
    pub last_block_height: i64,
    #[prost(bytes, tag="5")]
    #[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
    pub last_block_app_hash: std::vec::Vec<u8>,
}
/// nondeterministic
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseSetOption {
    #[prost(uint32, tag="1")]
    pub code: u32,
    /// bytes data = 2;
    #[prost(string, tag="3")]
    pub log: std::string::String,
    #[prost(string, tag="4")]
    pub info: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag="1")]
    pub consensus_params: ::std::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag="2")]
    pub validators: ::std::vec::Vec<ValidatorUpdate>,
    #[prost(bytes, tag="3")]
    pub app_hash: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseQuery {
    #[prost(uint32, tag="1")]
    pub code: u32,
    /// bytes data = 2; // use "value" instead.
    ///
    /// nondeterministic
    #[prost(string, tag="3")]
    pub log: std::string::String,
    /// nondeterministic
    #[prost(string, tag="4")]
    pub info: std::string::String,
    #[prost(int64, tag="5")]
    pub index: i64,
    #[prost(bytes, tag="6")]
    pub key: std::vec::Vec<u8>,
    #[prost(bytes, tag="7")]
    pub value: std::vec::Vec<u8>,
    #[prost(message, optional, tag="8")]
    pub proof_ops: ::std::option::Option<super::crypto::ProofOps>,
    #[prost(int64, tag="9")]
    pub height: i64,
    #[prost(string, tag="10")]
    pub codespace: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBeginBlock {
    #[prost(message, repeated, tag="1")]
    pub events: ::std::vec::Vec<Event>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag="1")]
    pub code: u32,
    #[prost(bytes, tag="2")]
    pub data: std::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag="3")]
    pub log: std::string::String,
    /// nondeterministic
    #[prost(string, tag="4")]
    pub info: std::string::String,
    #[prost(int64, tag="5")]
    pub gas_wanted: i64,
    #[prost(int64, tag="6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag="7")]
    pub events: ::std::vec::Vec<Event>,
    #[prost(string, tag="8")]
    pub codespace: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseDeliverTx {
    #[prost(uint32, tag="1")]
    pub code: u32,
    #[prost(bytes, tag="2")]
    pub data: std::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag="3")]
    pub log: std::string::String,
    /// nondeterministic
    #[prost(string, tag="4")]
    pub info: std::string::String,
    #[prost(int64, tag="5")]
    pub gas_wanted: i64,
    #[prost(int64, tag="6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag="7")]
    pub events: ::std::vec::Vec<Event>,
    #[prost(string, tag="8")]
    pub codespace: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEndBlock {
    #[prost(message, repeated, tag="1")]
    pub validator_updates: ::std::vec::Vec<ValidatorUpdate>,
    #[prost(message, optional, tag="2")]
    pub consensus_param_updates: ::std::option::Option<ConsensusParams>,
    #[prost(message, repeated, tag="3")]
    pub events: ::std::vec::Vec<Event>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCommit {
    /// reserve 1
    #[prost(bytes, tag="2")]
    pub data: std::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub retain_height: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseListSnapshots {
    #[prost(message, repeated, tag="1")]
    pub snapshots: ::std::vec::Vec<Snapshot>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseOfferSnapshot {
    #[prost(enumeration="response_offer_snapshot::Result", tag="1")]
    pub result: i32,
}
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
    #[prost(bytes, tag="1")]
    pub chunk: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseApplySnapshotChunk {
    #[prost(enumeration="response_apply_snapshot_chunk::Result", tag="1")]
    pub result: i32,
    /// Chunks to refetch and reapply
    #[prost(uint32, repeated, tag="2")]
    pub refetch_chunks: ::std::vec::Vec<u32>,
    /// Chunk senders to reject and ban
    #[prost(string, repeated, tag="3")]
    pub reject_senders: ::std::vec::Vec<std::string::String>,
}
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
    pub block: ::std::option::Option<BlockParams>,
    #[prost(message, optional, tag="2")]
    pub evidence: ::std::option::Option<super::types::EvidenceParams>,
    #[prost(message, optional, tag="3")]
    pub validator: ::std::option::Option<super::types::ValidatorParams>,
    #[prost(message, optional, tag="4")]
    pub version: ::std::option::Option<super::types::VersionParams>,
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
    pub votes: ::std::vec::Vec<VoteInfo>,
}
/// Event allows application developers to attach additional information to
/// ResponseBeginBlock, ResponseEndBlock, ResponseCheckTx and ResponseDeliverTx.
/// Later, transactions may be queried using these events.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag="1")]
    pub r#type: std::string::String,
    #[prost(message, repeated, tag="2")]
    pub attributes: ::std::vec::Vec<EventAttribute>,
}
/// EventAttribute is a single key-value pair, associated with an event.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(bytes, tag="1")]
    pub key: std::vec::Vec<u8>,
    #[prost(bytes, tag="2")]
    pub value: std::vec::Vec<u8>,
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
    #[prost(bytes, tag="3")]
    pub tx: std::vec::Vec<u8>,
    #[prost(message, optional, tag="4")]
    pub result: ::std::option::Option<ResponseDeliverTx>,
}
//----------------------------------------
// Blockchain Types

/// Validator
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The first 20 bytes of SHA256(public key)
    #[prost(bytes, tag="1")]
    pub address: std::vec::Vec<u8>,
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
    pub pub_key: ::std::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag="2")]
    pub power: i64,
}
/// VoteInfo
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteInfo {
    #[prost(message, optional, tag="1")]
    pub validator: ::std::option::Option<Validator>,
    #[prost(bool, tag="2")]
    pub signed_last_block: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    #[prost(enumeration="EvidenceType", tag="1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag="2")]
    pub validator: ::std::option::Option<Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag="3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag="4")]
    pub time: ::std::option::Option<super::super::google::protobuf::Timestamp>,
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
    #[prost(bytes, tag="4")]
    pub hash: std::vec::Vec<u8>,
    /// Arbitrary application metadata
    #[prost(bytes, tag="5")]
    pub metadata: std::vec::Vec<u8>,
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
