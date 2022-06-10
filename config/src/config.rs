//! Tendermint configuration file types (with serde parsers/serializers)
//!
//! This module contains types which correspond to the following config files:
//!
//! - `config.toml`: `config::TendermintConfig`
//! - `node_key.rs`: `config::node_key::NodeKey`
//! - `priv_validator_key.rs`: `config::priv_validator_key::PrivValidatorKey`

use alloc::collections::{btree_map, BTreeMap};
use core::{fmt, str::FromStr};
use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{de, de::Error as _, ser, Deserialize, Serialize};
use tendermint::{genesis::Genesis, node, Moniker, Timeout};

use crate::{
    net,
    node_key::NodeKey,
    prelude::*,
    serialization::{
        deserialize_comma_separated_list, deserialize_from_str, deserialize_optional_value,
        serialize_comma_separated_list, serialize_optional_value, serialize_to_str,
    },
    Error,
};

/// Tendermint `config.toml` file
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TendermintConfig {
    /// TCP or UNIX socket address of the ABCI application,
    /// or the name of an ABCI application compiled in with the Tendermint binary.
    pub proxy_app: net::Address,

    /// A custom human readable name for this node
    pub moniker: Moniker,

    /// Mode of Node: full | validator | seed
    /// * validator node
    ///   - all reactors
    ///   - with priv_validator_key.json, priv_validator_state.json
    /// * full node
    ///   - all reactors
    ///   - No priv_validator_key.json, priv_validator_state.json
    /// * seed node
    ///   - only P2P, PEX Reactor
    ///   - No priv_validator_key.json, priv_validator_state.json
    pub mode: NodeMode,

    /// Database backend: `goleveldb | cleveldb | boltdb | rocksdb | badgerdb`
    pub db_backend: DbBackend,

    /// Database directory
    pub db_dir: PathBuf,

    /// Output level for logging, including package level options
    pub log_level: LogLevel,

    /// Output format: 'plain' (colored text) or 'json'
    pub log_format: LogFormat,

    /// Path to the JSON file containing the initial validator set and other meta data
    pub genesis_file: PathBuf,

    /// Path to the JSON file containing the private key to use for node
    /// authentication in the p2p protocol
    pub node_key_file: PathBuf,

    /// Mechanism to connect to the ABCI application: socket | grpc
    pub abci: AbciMode,

    /// If `true`, query the ABCI app on connecting to a new peer
    /// so the app can decide if we should keep the connection or not
    pub filter_peers: bool,

    /// Private validator configuration options
    pub priv_validator: PrivValidatorConfig,

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

    /// statesync configuration options
    pub statesync: StatesyncConfig,

    /// blocksync configuration options
    pub blocksync: BlocksyncConfig,
}

impl TendermintConfig {
    /// Parse Tendermint `config.toml`
    pub fn parse_toml<T: AsRef<str>>(toml_string: T) -> Result<Self, Error> {
        let res = toml::from_str(toml_string.as_ref()).map_err(Error::toml)?;

        Ok(res)
    }

    /// Load `config.toml` from a file
    pub fn load_toml_file<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let toml_string = fs::read_to_string(path)
            .map_err(|e| Error::file_io(format!("{}", path.as_ref().display()), e))?;

        Self::parse_toml(toml_string)
    }

    /// Load `genesis.json` file from the configured location
    pub fn load_genesis_file(&self, home: impl AsRef<Path>) -> Result<Genesis, Error> {
        let path = home.as_ref().join(&self.genesis_file);
        let genesis_json = fs::read_to_string(&path)
            .map_err(|e| Error::file_io(format!("{}", path.display()), e))?;

        let res = serde_json::from_str(genesis_json.as_ref()).map_err(Error::serde_json)?;

        Ok(res)
    }

    /// Load `node_key.json` file from the configured location
    pub fn load_node_key(&self, home: impl AsRef<Path>) -> Result<NodeKey, Error> {
        let path = home.as_ref().join(&self.node_key_file);
        NodeKey::load_json_file(&path)
    }
}

/// The mode in which a node is to be started.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum NodeMode {
    /// Validator node:
    /// - all reactors
    /// - with priv_validator_key.json, priv_validator_state.json
    #[serde(rename = "validator")]
    Validator,

    /// Full node:
    /// - all reactors
    /// - No priv_validator_key.json, priv_validator_state.json
    #[serde(rename = "full")]
    Full,

    /// Seed node:
    /// - only P2P, PEX Reactor
    /// - No priv_validator_key.json, priv_validator_state.json
    #[serde(rename = "seed")]
    Seed,
}

impl Default for NodeMode {
    fn default() -> Self {
        Self::Validator
    }
}

/// Private validator configuration options
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct PrivValidatorConfig {
    /// Path to the JSON file containing the private key to use as a validator
    /// in the consensus protocol
    pub key_file: PathBuf,

    /// Path to the JSON file containing the last sign state of a validator
    pub state_file: PathBuf,

    /// TCP or UNIX socket address for Tendermint to listen on for connections
    /// from an external PrivValidator process when the listenAddr is prefixed
    /// with grpc instead of tcp it will use the gRPC Client
    #[serde(
        deserialize_with = "deserialize_optional_value",
        serialize_with = "serialize_optional_value"
    )]
    pub laddr: Option<net::Address>,

    /// Path to the client certificate generated while creating needed files for
    /// secure connection. If a remote validator address is provided but no
    /// certificate, the connection will be insecure
    pub client_certificate_file: PathBuf,

    /// Client key generated while creating certificates for secure connection
    pub client_key_file: PathBuf,

    /// Path to the Root Certificate Authority used to sign both client and
    /// server certificates
    pub root_ca_file: PathBuf,
}

/// Database backend
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DbBackend {
    /// GoLevelDB backend
    #[serde(rename = "goleveldb")]
    GoLevelDb,

    /// CLevelDB backend
    #[serde(rename = "cleveldb")]
    CLevelDb,

    /// BoltDB backend
    #[serde(rename = "boltdb")]
    BoltDb,

    /// RocksDB backend
    #[serde(rename = "rocksdb")]
    RocksDb,

    /// BadgerDB backend
    #[serde(rename = "badgerdb")]
    BadgerDb,
}

/// Loglevel configuration
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogLevel {
    /// A global log level
    pub global: Option<String>,
    components: BTreeMap<String, String>,
}

impl LogLevel {
    /// Get the setting for the given key. If not found, returns the global setting, if any.
    pub fn get<S>(&self, key: S) -> Option<&str>
    where
        S: AsRef<str>,
    {
        self.components
            .get(key.as_ref())
            .or(self.global.as_ref())
            .map(AsRef::as_ref)
    }

    /// Iterate over the levels. This doesn't include the global setting, if any.
    pub fn iter(&self) -> LogLevelIter<'_> {
        self.components.iter()
    }
}

/// Iterator over log levels
pub type LogLevelIter<'a> = btree_map::Iter<'a, String, String>;

impl FromStr for LogLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut global = None;
        let mut components = BTreeMap::new();

        for level in s.split(',') {
            let parts = level.split(':').collect::<Vec<_>>();

            if parts.len() == 1 {
                global = Some(parts[0].to_owned());
                continue;
            } else if parts.len() != 2 {
                return Err(Error::parse(format!("error parsing log level: {}", level)));
            }

            let key = parts[0].to_owned();
            let value = parts[1].to_owned();

            if components.insert(key, value).is_some() {
                return Err(Error::parse(format!(
                    "duplicate log level setting for: {}",
                    level
                )));
            }
        }

        Ok(LogLevel { global, components })
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(global) = &self.global {
            write!(f, "{}", global)?;
            if !self.components.is_empty() {
                write!(f, ",")?;
            }
        }
        for (i, (k, v)) in self.components.iter().enumerate() {
            write!(f, "{}:{}", k, v)?;

            if i < self.components.len() - 1 {
                write!(f, ",")?;
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let levels = String::deserialize(deserializer)?;
        Self::from_str(&levels).map_err(|e| D::Error::custom(format!("{}", e)))
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
    /// Deprecated gRPC in the RPC layer of Tendermint will be deprecated in
    /// 0.36.
    #[serde(
        deserialize_with = "deserialize_optional_value",
        serialize_with = "serialize_optional_value"
    )]
    pub grpc_laddr: Option<net::Address>,

    /// Maximum number of simultaneous GRPC connections. Does not include RPC
    /// (HTTP&WebSocket) connections. See `max_open_connections`.
    /// Deprecated gRPC  in the RPC layer of Tendermint will be deprecated in
    /// 0.36.
    pub grpc_max_open_connections: u64,

    /// Activate unsafe RPC commands like /dial-seeds and /unsafe-flush-mempool
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

    /// Maximum size of request body, in bytes
    pub max_body_bytes: u64,

    /// Maximum size of request header, in bytes
    pub max_header_bytes: u64,

    /// The name of a file containing certificate that is used to create the HTTPS server.
    #[serde(
        deserialize_with = "deserialize_optional_value",
        serialize_with = "serialize_optional_value"
    )]
    pub tls_cert_file: Option<PathBuf>,

    /// The name of a file containing matching private key that is used to create the HTTPS server.
    #[serde(
        deserialize_with = "deserialize_optional_value",
        serialize_with = "serialize_optional_value"
    )]
    pub tls_key_file: Option<PathBuf>,

    /// pprof listen address <https://golang.org/pkg/net/http/pprof>
    #[serde(
        deserialize_with = "deserialize_optional_value",
        serialize_with = "serialize_optional_value"
    )]
    pub pprof_laddr: Option<net::Address>,
}

/// Origin hosts allowed with CORS requests to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct P2PConfig {
    /// Enable the legacy p2p layer.
    pub use_legacy: bool,

    /// Select the p2p internal queue
    pub queue_type: P2PQueueType,

    /// Address to listen for incoming connections
    pub laddr: net::Address,

    /// Address to advertise to peers for them to dial If empty, will use the
    /// same port as the laddr, and will introspect on the listener or use UPnP
    /// to figure out the address. IP and port are required.
    #[serde(
        deserialize_with = "deserialize_optional_value",
        serialize_with = "serialize_optional_value"
    )]
    pub external_address: Option<net::Address>,

    /// Comma separated list of seed nodes to connect to. We only use these if
    /// we can’t connect to peers in the addrbook.
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub seeds: Vec<net::Address>,

    /// Comma separated list of peers to be added to the peer store on startup.
    /// Either `bootstrap_peers` or `persistent_peers` are needed for peer
    /// discovery
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub bootstrap_peers: Vec<net::Address>,

    /// Comma separated list of nodes to keep persistent connections to
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub persistent_peers: Vec<net::Address>,

    /// UPNP port forwarding
    pub upnp: bool,

    /// Path to address book.
    pub addr_book_file: PathBuf,

    /// Set `true` for strict address routability rules
    /// Set `false` for private or local networks
    pub addr_book_strict: bool,

    /// Maximum number of inbound peers.
    pub max_num_inbound_peers: u64,

    /// Maximum number of outbound peers to connect to, excluding persistent peers
    pub max_num_outbound_peers: u64,

    /// Maximum number of connections (inbound and outbound).
    pub max_connections: u64,

    /// List of node IDs, to which a connection will be (re)established ignoring any existing
    /// limits
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub unconditional_peer_ids: Vec<node::Id>,

    /// Maximum pause when redialing a persistent peer (if zero, exponential backoff is used)
    pub persistent_peers_max_dial_period: Timeout,

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

/// Makes it possible to configure which queue backend the P2P layer uses.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum P2PQueueType {
    #[serde(rename = "fifo")]
    FIFO,
    #[serde(rename = "priority")]
    Priority,
}

impl Default for P2PQueueType {
    fn default() -> Self {
        Self::Priority
    }
}

/// mempool configuration options
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MempoolConfig {
    /// Mempool version to use.
    pub version: MempoolVersion,

    /// Recheck enabled
    pub recheck: bool,

    /// Broadcast enabled
    pub broadcast: bool,

    /// Maximum number of transactions in the mempool
    pub size: u64,

    /// Limit the total size of all txs in the mempool.
    /// This only accounts for raw transactions (e.g. given 1MB transactions and
    /// `max_txs_bytes`=5MB, mempool will only accept 5 transactions).
    pub max_txs_bytes: u64,

    /// Size of the cache (used to filter transactions we saw earlier) in transactions
    pub cache_size: u64,

    /// Do not remove invalid transactions from the cache (default: false)
    /// Set to true if it's not possible for any invalid transaction to become valid
    /// again in the future.
    pub keep_invalid_txs_in_cache: bool,

    /// Maximum size of a single transaction.
    /// NOTE: the max size of a tx transmitted over the network is {max-tx-bytes}.
    pub max_tx_bytes: u64,

    /// Maximum size of a batch of transactions to send to a peer
    /// Including space needed by encoding (one varint per transaction).
    /// XXX: Unused due to <https://github.com/tendermint/tendermint/issues/5796>
    pub max_batch_bytes: u64,

    /// If non-zero, defines the maximum amount of time a transaction can exist
    /// for in the mempool.
    ///
    /// Note, if `ttl_num_blocks` is also defined, a transaction will be removed
    /// if it has existed in the mempool at least `ttl_num_blocks` number of
    /// blocks or if it's insertion time into the mempool is beyond
    /// `ttl_duration`.
    pub ttl_duration: Timeout,

    /// If non-zero, defines the maximum number of blocks a transaction can
    /// exist for in the mempool.
    ///
    /// Note, if `ttl_duration` is also defined, a transaction will be removed
    /// if it has existed in the mempool at least `ttl_num_blocks` number of
    /// blocks or if it's insertion time into the mempool is beyond
    /// `ttl_duration`.
    pub ttl_num_blocks: u64,
}

/// The mempool version to use.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum MempoolVersion {
    /// v0 of the mempool
    #[serde(rename = "v0")]
    Legacy,
    /// v1 of the mempool (the default)
    #[serde(rename = "v1")]
    Prioritized,
}

impl Default for MempoolVersion {
    fn default() -> Self {
        Self::Prioritized
    }
}

/// consensus configuration options
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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

    /// How many blocks to look back to check existence of the node's consensus votes before
    /// joining consensus When non-zero, the node will panic upon restart
    /// if the same consensus key was used to sign {double-sign-check-height} last blocks.
    /// So, validators should stop the state machine, wait for some blocks, and then restart the
    /// state machine to avoid panic.
    pub double_sign_check_height: u64,

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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TxIndexConfig {
    /// The backend database list to back the indexer. If list contains "null"
    /// or "", meaning no indexer service will be used.
    ///
    /// The application will set which txs to index. In some cases a node
    /// operator will be able to decide which txs to index based on
    /// configuration set in the application.
    #[serde(default)]
    pub indexer: Vec<TxIndexer>,

    /// The PostgreSQL connection configuration, the connection format:
    ///   `postgresql://<user>:<password>@<host>:<port>/<db>?<opts>`
    pub psql_conn: String,
}

/// What indexer to use for transactions
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TxIndexer {
    /// "null" - no indexer
    #[serde(rename = "null")]
    Null,

    /// "kv" (default) - the simplest possible indexer, backed by key-value
    /// storage (defaults to levelDB; see DBBackend).
    #[serde(rename = "kv")]
    Kv,

    /// "psql" - the indexer services backed by PostgreSQL.
    #[serde(rename = "psql")]
    PostgreSQL,
}

impl Default for TxIndexer {
    fn default() -> TxIndexer {
        TxIndexer::Kv
    }
}

/// instrumentation configuration options
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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

/// statesync configuration options
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct StatesyncConfig {
    /// State sync rapidly bootstraps a new node by discovering, fetching, and restoring a state
    /// machine snapshot from peers instead of fetching and replaying historical blocks.
    /// Requires some peers in the network to take and serve state machine snapshots. State
    /// sync is not attempted if the node has any local state (LastBlockHeight > 0). The node
    /// will have a truncated block history, starting from the height of the snapshot.
    pub enable: bool,

    /// State sync uses light client verification to verify state. This can be
    /// done either through the P2P layer or RPC layer. Set this to true to use
    /// the P2P layer. If false (default), RPC layer will be used.
    pub use_p2p: bool,

    /// If using RPC, at least two addresses need to be provided. They should be
    /// compatible with net.Dial, for example: "host.example.com:2125"
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub rpc_servers: Vec<String>,

    /// The height of a trusted block. Must be within the `trust_period`.
    pub trust_height: u64,

    /// The hash of a trusted block.
    pub trust_hash: String,

    /// Trust period. See `rpc_servers` above.
    pub trust_period: String,

    /// Time to spend discovering snapshots before initiating a restore.
    pub discovery_time: Timeout,

    /// Temporary directory for state sync snapshot chunks, defaults to the OS tempdir (typically
    /// /tmp). Will create a new, randomly named directory within, and remove it when done.
    pub temp_dir: String,

    /// The timeout duration before re-requesting a chunk, possibly from a
    /// different peer (default: 15 seconds).
    pub chunk_request_timeout: Timeout,

    /// The number of concurrent chunk and block fetchers to run (default: 4).
    #[serde(
        serialize_with = "serialize_to_str",
        deserialize_with = "deserialize_from_str"
    )]
    pub fetchers: u16,
}

/// blocksync configuration options
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct BlocksyncConfig {
    /// If this node is many blocks behind the tip of the chain, BlockSync
    /// allows them to catchup quickly by downloading blocks in parallel and
    /// verifying their commits
    pub enable: bool,

    /// Block Sync version to use:
    ///   1) "v0" (default) - the standard Block Sync implementation
    ///   2) "v2" - DEPRECATED, please use v0
    pub version: BlocksyncVersion,
}

/// Block sync version to use.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum BlocksyncVersion {
    /// The standard block sync implementation.
    #[serde(rename = "v0")]
    V0,
    /// Deprecated. Please use `V0`.
    #[serde(rename = "v1")]
    V2,
}

/// Rate at which bytes can be sent/received
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TransferRate(u64);

impl TransferRate {
    /// Get the trasfer rate in bytes per second
    pub fn bytes_per_sec(self) -> u64 {
        self.0
    }
}
