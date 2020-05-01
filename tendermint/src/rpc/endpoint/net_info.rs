//! `/net_info` endpoint JSONRPC wrapper

use crate::{channel::Channel, node, rpc, serializers, Time};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    net::IpAddr,
    time::Duration,
};

/// Request network information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::NetInfo
    }
}

/// Net info responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Are we presently listening?
    pub listening: bool,

    /// Active listeners
    pub listeners: Vec<Listener>,

    /// Number of connected peers
    #[serde(with = "serializers::primitives::string")]
    pub n_peers: u64,

    /// Peer information
    pub peers: Vec<PeerInfo>,
}

impl rpc::Response for Response {}

/// Listener information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Listener(String);

impl Display for Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Peer information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    /// Node information
    pub node_info: node::Info,

    /// Is this an outbound connection?
    pub is_outbound: bool,

    /// Connection status
    pub connection_status: ConnectionStatus,

    /// Remote IP address
    pub remote_ip: IpAddr,
}

/// Connection status information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionStatus {
    /// Duration of this connection
    #[serde(rename = "Duration", with = "serializers::timeduration::string")]
    pub duration: Duration,

    /// Send monitor
    #[serde(rename = "SendMonitor")]
    pub send_monitor: Monitor,

    /// Receive monitor
    #[serde(rename = "RecvMonitor")]
    pub recv_monitor: Monitor,

    /// Channels
    #[serde(rename = "Channels")]
    pub channels: Vec<Channel>,
}

/// Monitor
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Monitor {
    /// Is this monitor active?
    #[serde(rename = "Active")]
    pub active: bool,

    /// When the monitor started
    #[serde(rename = "Start")]
    pub start: Time,

    /// Duration of this monitor
    #[serde(rename = "Duration", with = "serializers::timeduration::string")]
    pub duration: Duration,

    /// Idle duration for this monitor
    #[serde(rename = "Idle", with = "serializers::timeduration::string")]
    pub idle: Duration,

    /// Bytes
    #[serde(rename = "Bytes", with = "serializers::primitives::string")]
    bytes: u64,

    /// Samples
    #[serde(rename = "Samples", with = "serializers::primitives::string")]
    samples: u64,

    /// Instant rate
    #[serde(rename = "InstRate", with = "serializers::primitives::string")]
    inst_rate: u64,

    /// Current rate
    #[serde(rename = "CurRate", with = "serializers::primitives::string")]
    cur_rate: u64,

    /// Average rate
    #[serde(rename = "AvgRate", with = "serializers::primitives::string")]
    avg_rate: u64,

    /// Peak rate
    #[serde(rename = "PeakRate", with = "serializers::primitives::string")]
    peak_rate: u64,

    /// Bytes remaining
    #[serde(rename = "BytesRem", with = "serializers::primitives::string")]
    bytes_rem: u64,

    /// Time remaining
    #[serde(rename = "TimeRem", with = "serializers::primitives::string")]
    time_rem: u64,

    /// Progress
    #[serde(rename = "Progress")]
    progress: u64,
}
