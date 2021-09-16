//! Tests for parsing configuration files.
//!
//! Test config files are located in the `tests/support/config` subdirectory.

#[cfg(test)]
use pretty_assertions::assert_eq;
use std::{fs, path::PathBuf, time::Duration};
use tendermint::node;
use tendermint_config::*;

/// Read a fixture file from the `support/config` directory
fn read_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("./tests/support/config/").join(name)).unwrap()
}

/// Parse an example `config.toml` file to a `TendermintConfig` struct
#[allow(clippy::cognitive_complexity)]
#[test]
fn config_toml_parser() {
    let config_toml = read_fixture("config.toml");
    let config = TendermintConfig::parse_toml(&config_toml).unwrap();

    // main base config options

    assert_eq!(
        config.proxy_app,
        "tcp://127.0.0.1:26658".parse::<net::Address>().unwrap()
    );
    assert_eq!(config.moniker.as_ref(), "technodrome");
    assert!(config.fast_sync);
    assert_eq!(config.db_backend, DbBackend::GoLevelDb);
    assert_eq!(config.db_dir, PathBuf::from("data"));
    assert_eq!(config.log_level.global, Some("info".to_string()));
    assert_eq!(config.log_level.get("main"), Some("info"));
    assert_eq!(config.log_level.get("state"), Some("info"));
    assert_eq!(config.log_level.get("*"), Some("info"));
    assert_eq!(config.log_format, LogFormat::Plain);
    assert_eq!(config.genesis_file, PathBuf::from("config/genesis.json"));
    assert_eq!(
        config.priv_validator_key_file,
        Some(PathBuf::from("config/priv_validator_key.json"))
    );
    assert_eq!(
        config.priv_validator_state_file,
        PathBuf::from("data/priv_validator_state.json")
    );
    assert_eq!(config.priv_validator_laddr, None);
    assert_eq!(config.node_key_file, PathBuf::from("config/node_key.json"));
    assert_eq!(config.abci, AbciMode::Socket);
    assert!(!config.filter_peers);

    // rpc server configuration options

    let rpc = &config.rpc;
    assert_eq!(
        rpc.laddr,
        "tcp://0.0.0.0:26657".parse::<net::Address>().unwrap()
    );
    assert!(rpc.cors_allowed_origins.is_empty());
    assert_eq!(rpc.cors_allowed_methods.len(), 3);
    assert_eq!(rpc.cors_allowed_methods[0].as_ref(), "HEAD");
    assert_eq!(rpc.cors_allowed_methods[1].as_ref(), "GET");
    assert_eq!(rpc.cors_allowed_methods[2].as_ref(), "POST");
    assert_eq!(rpc.cors_allowed_headers.len(), 5);
    assert_eq!(rpc.cors_allowed_headers[0].as_ref(), "Origin");
    assert_eq!(rpc.cors_allowed_headers[1].as_ref(), "Accept");
    assert_eq!(rpc.cors_allowed_headers[2].as_ref(), "Content-Type");
    assert_eq!(rpc.cors_allowed_headers[3].as_ref(), "X-Requested-With");
    assert_eq!(rpc.cors_allowed_headers[4].as_ref(), "X-Server-Time");
    assert_eq!(rpc.grpc_laddr, None);
    assert_eq!(rpc.grpc_max_open_connections, 900);
    assert!(!rpc.unsafe_commands);
    assert_eq!(rpc.max_open_connections, 900);
    assert_eq!(rpc.max_subscription_clients, 100);
    assert_eq!(rpc.max_subscriptions_per_client, 5);
    assert_eq!(*rpc.timeout_broadcast_tx_commit, Duration::from_secs(10));
    assert_eq!(rpc.tls_cert_file, None);
    assert_eq!(rpc.tls_key_file, None);

    // peer to peer configuration options

    let p2p = &config.p2p;
    assert_eq!(
        p2p.laddr,
        "tcp://0.0.0.0:26656".parse::<net::Address>().unwrap()
    );
    assert_eq!(p2p.external_address, None);
    assert_eq!(p2p.seeds.len(), 2);
    assert_eq!(
        p2p.seeds[0],
        "tcp://c2e1bde78877975b31e6f06e77da200a38048e2b@seed-1.example.com:26656"
            .parse::<net::Address>()
            .unwrap()
    );
    assert_eq!(
        p2p.seeds[1],
        "tcp://0eafed3e9e76f626a299e1b8a79454fffe9ca83c@seed-2.example.com:26656"
            .parse::<net::Address>()
            .unwrap()
    );
    assert_eq!(p2p.persistent_peers.len(), 2);
    assert_eq!(
        p2p.persistent_peers[0],
        "tcp://70d834561f91613153e4a873f01a2cbbf1b9678d@1.2.3.4:26656"
            .parse::<net::Address>()
            .unwrap()
    );
    assert_eq!(
        p2p.persistent_peers[1],
        "tcp://f68ed33a0baa0c734a939a9e60659566adc725cd@peer-2.example.com:26656"
            .parse::<net::Address>()
            .unwrap()
    );
    assert!(!p2p.upnp);
    assert_eq!(p2p.addr_book_file, PathBuf::from("config/addrbook.json"));
    assert!(p2p.addr_book_strict);
    assert_eq!(p2p.max_num_inbound_peers, 40);
    assert_eq!(p2p.max_num_outbound_peers, 10);
    assert_eq!(*p2p.flush_throttle_timeout, Duration::from_millis(100));
    assert_eq!(p2p.max_packet_msg_payload_size, 1024);
    assert_eq!(p2p.send_rate.bytes_per_sec(), 5_120_000);
    assert_eq!(p2p.recv_rate.bytes_per_sec(), 5_120_000);
    assert!(p2p.pex);
    assert!(!p2p.seed_mode);
    assert_eq!(p2p.private_peer_ids.len(), 3);
    assert_eq!(
        p2p.private_peer_ids[0],
        "8112E5C5AB6A48ADCC0E875D58A4264A2639F6A8"
            .parse::<node::Id>()
            .unwrap()
    );
    assert_eq!(
        p2p.private_peer_ids[1],
        "3D1B9086E48C7BDF7F0D766351EED812A75DE500"
            .parse::<node::Id>()
            .unwrap()
    );
    assert_eq!(
        p2p.private_peer_ids[2],
        "A7306AEE50627E68177A002BADD3BA4A45301AD4"
            .parse::<node::Id>()
            .unwrap()
    );
    assert!(!p2p.allow_duplicate_ip);
    assert_eq!(*p2p.handshake_timeout, Duration::from_secs(20));
    assert_eq!(*p2p.dial_timeout, Duration::from_secs(3));

    // mempool configuration options

    let mempool = &config.mempool;
    assert!(mempool.recheck);
    assert!(mempool.broadcast);
    assert_eq!(mempool.wal_dir, None);
    assert_eq!(mempool.size, 5000);
    assert_eq!(mempool.max_txs_bytes, 1_073_741_824);
    assert_eq!(mempool.cache_size, 10000);

    // consensus configuration options

    let consensus = &config.consensus;
    assert_eq!(consensus.wal_file, PathBuf::from("data/cs.wal/wal"));
    assert_eq!(*consensus.timeout_propose, Duration::from_secs(3));
    assert_eq!(*consensus.timeout_propose_delta, Duration::from_millis(500));
    assert_eq!(*consensus.timeout_prevote, Duration::from_secs(1));
    assert_eq!(*consensus.timeout_prevote_delta, Duration::from_millis(500));
    assert_eq!(*consensus.timeout_precommit, Duration::from_secs(1));
    assert_eq!(
        *consensus.timeout_precommit_delta,
        Duration::from_millis(500)
    );
    assert_eq!(*consensus.timeout_commit, Duration::from_secs(5));
    assert!(!consensus.skip_timeout_commit);
    assert_eq!(
        *consensus.create_empty_blocks_interval,
        Duration::from_secs(0)
    );
    assert_eq!(
        *consensus.peer_gossip_sleep_duration,
        Duration::from_millis(100)
    );
    assert_eq!(
        *consensus.peer_query_maj23_sleep_duration,
        Duration::from_secs(2)
    );

    // transactions indexer configuration options

    let tx_index = &config.tx_index;
    assert_eq!(tx_index.indexer, TxIndexer::Kv);

    // instrumentation configuration options

    let instrumentation = &config.instrumentation;
    assert!(!instrumentation.prometheus);
    assert_eq!(instrumentation.prometheus_listen_addr, ":26660");
    assert_eq!(instrumentation.max_open_connections, 3);
    assert_eq!(instrumentation.namespace, "tendermint");
}

/// Parse an example `node_key.json` file to a `NodeKey` struct
#[test]
fn node_key_parser() {
    let raw_node_key = read_fixture("node_key.json");
    let node_key = NodeKey::parse_json(&raw_node_key).unwrap();
    assert_eq!(
        node_key.node_id().to_string(),
        "1a7b6bcf3d6fb055ab3aebca415847531b626699"
    );
}

/// Parse an example `priv_validator_key.json` to a `PrivValidatorKey` struct
#[test]
fn priv_validator_json_parser() {
    let raw_priv_validator_key = read_fixture("priv_validator_key.json");
    let priv_validator_key = PrivValidatorKey::parse_json(&raw_priv_validator_key).unwrap();
    assert_eq!(
        priv_validator_key.consensus_pubkey().to_hex(),
        "F26BF4B2A2E84CEB7A53C3F1AE77408779B20064782FBADBDF0E365959EE4534"
    );
}

/// Parse an example `config.toml` file to a `TendermintConfig` struct, then
/// serialize it and parse again.
#[test]
fn parsing_roundtrip() {
    let config_toml = read_fixture("config.toml");
    let config = TendermintConfig::parse_toml(&config_toml).unwrap();

    let written_config_toml = toml::to_string(&config).unwrap();
    let written_config = TendermintConfig::parse_toml(&written_config_toml).unwrap();

    assert_eq!(
        config, written_config,
        "written config {}",
        written_config_toml
    );
}
