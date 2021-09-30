//! `/net_info` endpoint JSON-RPC wrapper

use core::fmt::{self, Display};
use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use tendermint::{channel::Channel, node, serializers, Time};

use crate::prelude::*;

/// Request network information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::NetInfo
    }
}

impl crate::SimpleRequest for Request {}

/// Net info responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Are we presently listening?
    pub listening: bool,

    /// Active listeners
    pub listeners: Vec<Listener>,

    /// Number of connected peers
    #[serde(with = "serializers::from_str")]
    pub n_peers: u64,

    /// Peer information
    pub peers: Vec<PeerInfo>,
}

impl crate::Response for Response {}

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
    #[serde(rename = "Duration", with = "serializers::time_duration")]
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
    #[serde(rename = "Duration", with = "serializers::time_duration")]
    pub duration: Duration,

    /// Idle duration for this monitor
    #[serde(rename = "Idle", with = "serializers::time_duration")]
    pub idle: Duration,

    /// Bytes
    #[serde(rename = "Bytes", with = "serializers::from_str")]
    pub bytes: u64,

    /// Samples
    #[serde(rename = "Samples", with = "serializers::from_str")]
    pub samples: u64,

    /// Instant rate
    #[serde(rename = "InstRate", with = "serializers::from_str")]
    pub inst_rate: u64,

    /// Current rate
    #[serde(rename = "CurRate", with = "serializers::from_str")]
    pub cur_rate: u64,

    /// Average rate
    #[serde(rename = "AvgRate", with = "serializers::from_str")]
    pub avg_rate: u64,

    /// Peak rate
    #[serde(rename = "PeakRate", with = "serializers::from_str")]
    pub peak_rate: u64,

    /// Bytes remaining
    #[serde(rename = "BytesRem", with = "serializers::from_str")]
    pub bytes_rem: u64,

    /// Time remaining
    #[serde(rename = "TimeRem", with = "serializers::from_str")]
    pub time_rem: u64,

    /// Progress
    #[serde(rename = "Progress")]
    pub progress: u64,
}
