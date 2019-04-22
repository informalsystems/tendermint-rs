//! Integration tests

/// RPC integration tests
///
/// NOTE: health is tested implicitly when the initial client is created
#[cfg(all(feature = "integration", feature = "rpc"))]
mod rpc {
    use tendermint::rpc::Client;

    /// Get the address of the local node
    #[cfg(all(feature = "integration", feature = "rpc"))]
    pub fn localhost_rpc_client() -> Client {
        Client::new(&"tcp://127.0.0.1:26657".parse().unwrap()).unwrap()
    }

    /// `/abci_info` endpoint
    #[test]
    fn abci_info() {
        let abci_info = localhost_rpc_client().abci_info().unwrap();

        // TODO(tarcieri): integration testing support for non-gaia apps
        assert_eq!(&abci_info.data, "GaiaApp");
    }

    /// `/block` endpoint
    #[test]
    fn block() {
        let height = 1u64;
        let block_info = localhost_rpc_client().block(height).unwrap();
        assert_eq!(block_info.block_meta.header.height.value(), height);
    }

    /// `/blockchain` endpoint
    #[test]
    fn blockchain() {
        let blockchain_info = localhost_rpc_client().blockchain(1u64, 10u64).unwrap();
        assert_eq!(blockchain_info.block_metas.len(), 10);
    }

    /// `/commit` endpoint
    #[test]
    fn commit() {
        let height = 1u64;
        let commit_info = localhost_rpc_client().block(height).unwrap();
        assert_eq!(commit_info.block_meta.header.height.value(), height);
    }

    /// `/genesis` endpoint
    #[test]
    fn genesis() {
        let genesis = localhost_rpc_client().genesis().unwrap();
        assert_eq!(
            genesis.consensus_params.validator.pub_key_types[0].to_string(),
            "ed25519"
        );
    }

    /// `/net_info` endpoint integration test
    #[test]
    fn net_info() {
        let net_info = localhost_rpc_client().net_info().unwrap();
        assert!(net_info.listening);
    }

    /// `/status` endpoint integration test
    #[test]
    fn status_integration() {
        let status = localhost_rpc_client().status().unwrap();

        // For lack of better things to test
        assert_eq!(
            status.validator_info.voting_power.value(),
            0,
            "don't integration test against a validator"
        );
    }
}
