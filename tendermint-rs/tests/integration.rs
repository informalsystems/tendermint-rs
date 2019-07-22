//! Integration tests

/// RPC integration tests.
///
/// These are all ignored by default, since they test against running `gaiad`.
/// They can be run using:
///
/// ```
/// cargo test -- --ignored
/// ```
#[cfg(all(feature = "rpc"))]
mod rpc {
    use tendermint::rpc::Client;

    /// Get the address of the local node
    pub fn localhost_rpc_client() -> Client {
        Client::new(&"tcp://127.0.0.1:26657".parse().unwrap()).unwrap()
    }

    /// `/abci_info` endpoint
    #[test]
    #[ignore]
    fn abci_info() {
        let abci_info = localhost_rpc_client().abci_info().unwrap();
        assert_eq!(&abci_info.data, "GaiaApp");
    }

    /// `/abci_query` endpoint
    #[test]
    #[ignore]
    fn abci_query() {
        // TODO(tarcieri): write integration test for this endpoint
    }

    /// `/block` endpoint
    #[test]
    #[ignore]
    fn block() {
        let height = 1u64;
        let block_info = localhost_rpc_client().block(height).unwrap();
        assert_eq!(block_info.block_meta.header.height.value(), height);
    }

    /// `/block_results` endpoint
    #[test]
    #[ignore]
    fn block_results() {
        let height = 1u64;
        let block_results = localhost_rpc_client().block_results(height).unwrap();
        assert_eq!(block_results.height.value(), height);
    }

    /// `/blockchain` endpoint
    #[test]
    #[ignore]
    fn blockchain() {
        let blockchain_info = localhost_rpc_client().blockchain(1u64, 10u64).unwrap();
        assert_eq!(blockchain_info.block_metas.len(), 10);
    }

    /// `/commit` endpoint
    #[test]
    #[ignore]
    fn commit() {
        let height = 1u64;
        let commit_info = localhost_rpc_client().block(height).unwrap();
        assert_eq!(commit_info.block_meta.header.height.value(), height);
    }

    /// `/genesis` endpoint
    #[test]
    #[ignore]
    fn genesis() {
        let genesis = localhost_rpc_client().genesis().unwrap();
        assert_eq!(
            genesis.consensus_params.validator.pub_key_types[0].to_string(),
            "ed25519"
        );
    }

    /// `/net_info` endpoint integration test
    #[test]
    #[ignore]
    fn net_info() {
        let net_info = localhost_rpc_client().net_info().unwrap();
        assert!(net_info.listening);
    }

    /// `/status` endpoint integration test
    #[test]
    #[ignore]
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
