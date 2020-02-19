//! Tendermint configuration file types (with serde parsers/serializers)
//!
//! This module contains types which correspond to the following config files:
//!
//! - `config.toml`: `config::TendermintConfig`
//! - `node_key.rs`: `config::node_key::NodeKey`
//! - `priv_validator_key.rs`: `config::priv_validator_key::PrivValidatorKey`

mod node_key;
mod priv_validator_key;

pub use self::{node_key::NodeKey, priv_validator_key::PrivValidatorKey};

use crate::{
    abci::tag,
    error::{Error, Kind},
    genesis::Genesis,
    net, node, Moniker, Timeout,
};
use anomaly::{fail, format_err};
use serde::{de, de::Error as _, ser, Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Tendermint `config.toml` file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TendermintConfig {
    /// TCP or UNIX socket address of the ABCI application,
    /// or the name of an ABCI application compiled in with the Tendermint binary.
    pub proxy_app: net::Address,

    /// A custom human readable name for this node
    pub moniker: Moniker,

    /// If this node is many blocks behind the tip of the chain, FastSync
    /// allows them to catchup quickly by downloading blocks in parallel
    /// and verifying their commits
    pub fast_sync: bool,

    /// Database backend: `leveldb | memdb | cleveldb`
    pub db_backend: DbBackend,

    /// Database directory
    pub db_dir: PathBuf,

    /// Output level for logging, including package level options
    pub log_level: LogLevel,

    /// Output format: 'plain' (colored text) or 'json'
    pub log_format: LogFormat,

    /// Path to the JSON file containing the initial validator set and other meta data
    pub genesis_file: PathBuf,

    /// Path to the JSON file containing the private key to use as a validator in the consensus
    /// protocol
    pub priv_validator_key_file: Option<PathBuf>,

    /// Path to the JSON file containing the last sign state of a validator
    pub priv_validator_state_file: PathBuf,

    /// TCP or UNIX socket address for Tendermint to listen on for
    /// connections from an external PrivValidator process
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub priv_validator_laddr: Option<net::Address>,

    /// Path to the JSON file containing the private key to use for node authentication in the p2p
    /// protocol
    pub node_key_file: PathBuf,

    /// Mechanism to connect to the ABCI application: socket | grpc
    pub abci: AbciMode,

    /// TCP or UNIX socket address for the profiling server to listen on
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub prof_laddr: Option<net::Address>,

    /// If `true`, query the ABCI app on connecting to a new peer
    /// so the app can decide if we should keep the connection or not
    pub filter_peers: bool,

    /// rpc server configuration options
    pub rpc: RpcConfig,

    /// peer to peer configuration options
    pub p2p: P2PConfig,

    /// mempool configuration options
    pub mempool: MempoolConfig,

    /// consensus configuration options
    pub consensus: ConsensusConfig,

    /// transactions indexer configuration options
    pub tx_index: TxIndexConfig,

    /// instrumentation configuration options
    pub instrumentation: InstrumentationConfig,
}

impl TendermintConfig {
    /// Parse Tendermint `config.toml`
    pub fn parse_toml<T: AsRef<str>>(toml_string: T) -> Result<Self, Error> {
        Ok(toml::from_str(toml_string.as_ref())?)
    }

    /// Load `config.toml` from a file
    pub fn load_toml_file<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let toml_string = fs::read_to_string(path).map_err(|e| {
            format_err!(
                Kind::Parse,
                "couldn't open {}: {}",
                path.as_ref().display(),
                e
            )
        })?;

        Self::parse_toml(toml_string)
    }

    /// Load `genesis.json` file from the configured location
    pub fn load_genesis_file(&self, home: impl AsRef<Path>) -> Result<Genesis, Error> {
        let path = home.as_ref().join(&self.genesis_file);
        let genesis_json = fs::read_to_string(&path)
            .map_err(|e| format_err!(Kind::Parse, "couldn't open {}: {}", path.display(), e))?;

        Ok(serde_json::from_str(genesis_json.as_ref())?)
    }

    /// Load `node_key.json` file from the configured location
    pub fn load_node_key(&self, home: impl AsRef<Path>) -> Result<NodeKey, Error> {
        let path = home.as_ref().join(&self.node_key_file);
        NodeKey::load_json_file(&path)
    }
}

/// Database backend
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DbBackend {
    /// LevelDB backend
    #[serde(rename = "leveldb")]
    LevelDb,

    /// MemDB backend
    #[serde(rename = "memdb")]
    MemDb,

    /// CLevelDB backend
    #[serde(rename = "cleveldb")]
    CLevelDb,
}

/// Loglevel configuration
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogLevel(BTreeMap<String, String>);

impl LogLevel {
    /// Get the setting for the given key
    pub fn get<S>(&self, key: S) -> Option<&str>
    where
        S: AsRef<str>,
    {
        self.0.get(key.as_ref()).map(AsRef::as_ref)
    }

    /// Iterate over the levels
    pub fn iter(&self) -> LogLevelIter<'_> {
        self.0.iter()
    }
}

/// Iterator over log levels
pub type LogLevelIter<'a> = std::collections::btree_map::Iter<'a, String, String>;

impl FromStr for LogLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut levels = BTreeMap::new();

        for level in s.split(',') {
            let parts = level.split(':').collect::<Vec<_>>();

            if parts.len() != 2 {
                fail!(Kind::Parse, "error parsing log level: {}", level);
            }

            let key = parts[0].to_owned();
            let value = parts[1].to_owned();

            if levels.insert(key, value).is_some() {
                fail!(Kind::Parse, "duplicate log level setting for: {}", level);
            }
        }

        Ok(LogLevel(levels))
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (k, v)) in self.0.iter().enumerate() {
            write!(f, "{}:{}", k, v)?;

            if i < self.0.len() - 1 {
                write!(f, ",")?;
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let levels = String::deserialize(deserializer)?;
        Ok(Self::from_str(&levels).map_err(|e| D::Error::custom(format!("{}", e)))?)
    }
}

impl Serialize for LogLevel {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

/// Logging format
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LogFormat {
    /// Plain (colored text)
    #[serde(rename = "plain")]
    Plain,

    /// JSON
    #[serde(rename = "json")]
    Json,
}

/// Mechanism to connect to the ABCI application: socket | grpc
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AbciMode {
    /// Socket
    #[serde(rename = "socket")]
    Socket,

    /// GRPC
    #[serde(rename = "grpc")]
    Grpc,
}

/// Tendermint `config.toml` file's `[rpc]` section
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpcConfig {
    /// TCP or UNIX socket address for the RPC server to listen on
    pub laddr: net::Address,

    /// A list of origins a cross-domain request can be executed from
    /// Default value `[]` disables cors support
    /// Use `["*"]` to allow any origin
    pub cors_allowed_origins: Vec<CorsOrigin>,

    /// A list of methods the client is allowed to use with cross-domain requests
    pub cors_allowed_methods: Vec<CorsMethod>,

    /// A list of non simple headers the client is allowed to use with cross-domain requests
    pub cors_allowed_headers: Vec<CorsHeader>,

    /// TCP or UNIX socket address for the gRPC server to listen on
    /// NOTE: This server only supports `/broadcast_tx_commit`
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub grpc_laddr: Option<net::Address>,

    /// Maximum number of simultaneous GRPC connections.
    /// Does not include RPC (HTTP&WebSocket) connections. See `max_open_connections`.
    pub grpc_max_open_connections: u64,

    /// Activate unsafe RPC commands like `/dial_seeds` and `/unsafe_flush_mempool`
    #[serde(rename = "unsafe")]
    pub unsafe_commands: bool,

    /// Maximum number of simultaneous connections (including WebSocket).
    /// Does not include gRPC connections. See `grpc_max_open_connections`.
    pub max_open_connections: u64,

    /// Maximum number of unique clientIDs that can `/subscribe`.
    pub max_subscription_clients: u64,

    /// Maximum number of unique queries a given client can `/subscribe` to.
    pub max_subscriptions_per_client: u64,

    /// How long to wait for a tx to be committed during `/broadcast_tx_commit`.
    pub timeout_broadcast_tx_commit: Timeout,

    /// The name of a file containing certificate that is used to create the HTTPS server.
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub tls_cert_file: Option<PathBuf>,

    /// The name of a file containing matching private key that is used to create the HTTPS server.
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub tls_key_file: Option<PathBuf>,
}

/// Origin hosts allowed with CORS requests to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsOrigin(String);

impl AsRef<str> for CorsOrigin {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for CorsOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// HTTP methods allowed with CORS requests to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsMethod(String);

impl AsRef<str> for CorsMethod {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for CorsMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// HTTP headers allowed to be sent via CORS to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsHeader(String);

impl AsRef<str> for CorsHeader {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for CorsHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// peer to peer configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct P2PConfig {
    /// Address to listen for incoming connections
    pub laddr: net::Address,

    /// Address to advertise to peers for them to dial
    /// If empty, will use the same port as the laddr,
    /// and will introspect on the listener or use UPnP
    /// to figure out the address.
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub external_address: Option<net::Address>,

    /// Comma separated list of seed nodes to connect to
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub seeds: Vec<net::Address>,

    /// Comma separated list of nodes to keep persistent connections to
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub persistent_peers: Vec<net::Address>,

    /// UPNP port forwarding
    pub upnp: bool,

    /// Path to address book
    pub addr_book_file: PathBuf,

    /// Set `true` for strict address routability rules
    /// Set `false` for private or local networks
    pub addr_book_strict: bool,

    /// Maximum number of inbound peers
    pub max_num_inbound_peers: u64,

    /// Maximum number of outbound peers to connect to, excluding persistent peers
    pub max_num_outbound_peers: u64,

    /// Time to wait before flushing messages out on the connection
    pub flush_throttle_timeout: Timeout,

    /// Maximum size of a message packet payload, in bytes
    pub max_packet_msg_payload_size: u64,

    /// Rate at which packets can be sent, in bytes/second
    pub send_rate: TransferRate,

    /// Rate at which packets can be received, in bytes/second
    pub recv_rate: TransferRate,

    /// Set `true` to enable the peer-exchange reactor
    pub pex: bool,

    /// Seed mode, in which node constantly crawls the network and looks for
    /// peers. If another node asks it for addresses, it responds and disconnects.
    ///
    /// Does not work if the peer-exchange reactor is disabled.
    pub seed_mode: bool,

    /// Comma separated list of peer IDs to keep private (will not be gossiped to other peers)
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub private_peer_ids: Vec<node::Id>,

    /// Toggle to disable guard against peers connecting from the same ip.
    pub allow_duplicate_ip: bool,

    /// Handshake timeout
    pub handshake_timeout: Timeout,

    /// Timeout when dialing other peers
    pub dial_timeout: Timeout,
}

/// mempool configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolConfig {
    /// Recheck enabled
    pub recheck: bool,

    /// Broadcast enabled
    pub broadcast: bool,

    /// WAL dir
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub wal_dir: Option<PathBuf>,

    /// Maximum number of transactions in the mempool
    pub size: u64,

    /// Limit the total size of all txs in the mempool.
    /// This only accounts for raw transactions (e.g. given 1MB transactions and
    /// `max_txs_bytes`=5MB, mempool will only accept 5 transactions).
    pub max_txs_bytes: u64,

    /// Size of the cache (used to filter transactions we saw earlier) in transactions
    pub cache_size: u64,
}

/// consensus configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsensusConfig {
    /// Path to WAL file
    pub wal_file: PathBuf,

    /// Propose timeout
    pub timeout_propose: Timeout,

    /// Propose timeout delta
    pub timeout_propose_delta: Timeout,

    /// Prevote timeout
    pub timeout_prevote: Timeout,

    /// Prevote timeout delta
    pub timeout_prevote_delta: Timeout,

    /// Precommit timeout
    pub timeout_precommit: Timeout,

    /// Precommit timeout delta
    pub timeout_precommit_delta: Timeout,

    /// Commit timeout
    pub timeout_commit: Timeout,

    /// Make progress as soon as we have all the precommits (as if TimeoutCommit = 0)
    pub skip_timeout_commit: bool,

    /// EmptyBlocks mode
    pub create_empty_blocks: bool,

    /// Interval between empty blocks
    pub create_empty_blocks_interval: Timeout,

    /// Reactor sleep duration
    pub peer_gossip_sleep_duration: Timeout,

    /// Reactor query sleep duration
    pub peer_query_maj23_sleep_duration: Timeout,
}

/// transactions indexer configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxIndexConfig {
    /// What indexer to use for transactions
    #[serde(default)]
    pub indexer: TxIndexer,

    /// Comma-separated list of tags to index (by default the only tag is `tx.hash`)
    // TODO(tarcieri): switch to `tendermint::abci::Tag`
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub index_tags: Vec<tag::Key>,

    /// When set to true, tells indexer to index all tags (predefined tags:
    /// `tx.hash`, `tx.height` and all tags from DeliverTx responses).
    pub index_all_tags: bool,
}

/// What indexer to use for transactions
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TxIndexer {
    /// "null"
    // TODO(tarcieri): use an `Option` type here?
    #[serde(rename = "null")]
    Null,

    /// "kv" (default) - the simplest possible indexer, backed by key-value storage (defaults to
    /// levelDB; see DBBackend).
    #[serde(rename = "kv")]
    Kv,
}

impl Default for TxIndexer {
    fn default() -> TxIndexer {
        TxIndexer::Kv
    }
}

/// instrumentation configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstrumentationConfig {
    /// When `true`, Prometheus metrics are served under /metrics on
    /// PrometheusListenAddr.
    pub prometheus: bool,

    /// Address to listen for Prometheus collector(s) connections
    // TODO(tarcieri): parse to `tendermint::net::Addr`
    pub prometheus_listen_addr: String,

    /// Maximum number of simultaneous connections.
    pub max_open_connections: u64,

    /// Instrumentation namespace
    pub namespace: String,
}

/// Rate at which bytes can be sent/received
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct TransferRate(u64);

impl TransferRate {
    /// Get the trasfer rate in bytes per second
    pub fn bytes_per_sec(self) -> u64 {
        self.0
    }
}

/// Deserialize `Option<T: FromStr>` where an empty string indicates `None`
fn deserialize_optional_value<'de, D, T, E>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr<Err = E>,
    E: fmt::Display,
{
    let string = String::deserialize(deserializer)?;

    if string.is_empty() {
        return Ok(None);
    }

    string
        .parse()
        .map(Some)
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Deserialize a comma separated list of types that impl `FromStr` as a `Vec`
fn deserialize_comma_separated_list<'de, D, T, E>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr<Err = E>,
    E: fmt::Display,
{
    let mut result = vec![];
    let string = String::deserialize(deserializer)?;

    if string.is_empty() {
        return Ok(result);
    }

    for item in string.split(',') {
        result.push(
            item.parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        );
    }

    Ok(result)
}

/// Serialize a comma separated list types that impl `ToString`
fn serialize_comma_separated_list<S, T>(list: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    T: ToString,
{
    let str_list = list.iter().map(|addr| addr.to_string()).collect::<Vec<_>>();
    str_list.join(",").serialize(serializer)
}
