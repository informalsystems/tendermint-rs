//! Integration tests

/// RPC integration tests.
///
/// These are all ignored by default, since they test against running
/// `tendermint node --proxy_app=kvstore`. They can be run using:
///
/// ```
/// cargo test -- --ignored
/// ```
mod rpc {
    use core::future::Future;
    use tendermint::rpc::Client;
    use tokio::runtime::Builder;

    fn block_on<F: Future>(future: F) -> F::Output {
        Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    /// Get the address of the local node
    pub fn localhost_rpc_client() -> Client {
        block_on(Client::new(&"tcp://127.0.0.1:26657".parse().unwrap())).unwrap()
    }

    /// `/abci_info` endpoint
    #[test]
    #[ignore]
    fn abci_info() {
        let abci_info = block_on(localhost_rpc_client().abci_info()).unwrap();
        assert_eq!(&abci_info.data, "GaiaApp");
    }

    /// `/abci_query` endpoint
    #[test]
    #[ignore]
    fn abci_query() {
        let key = "unpopulated_key".parse().unwrap();
        let abci_query =
            block_on(localhost_rpc_client().abci_query(Some(key), vec![], None, false)).unwrap();
        assert_eq!(abci_query.key.as_ref().unwrap(), &Vec::<u8>::new());
        assert_eq!(abci_query.value.as_ref(), None);
    }

    /// `/block` endpoint
    #[test]
    #[ignore]
    fn block() {
        let height = 1u64;
        let block_info = block_on(localhost_rpc_client().block(height)).unwrap();
        assert_eq!(block_info.block_meta.header.height.value(), height);
    }

    /// `/block_results` endpoint
    #[test]
    #[ignore]
    fn block_results() {
        let height = 1u64;
        let block_results = block_on(localhost_rpc_client().block_results(height)).unwrap();
        assert_eq!(block_results.height.value(), height);
    }

    /// `/blockchain` endpoint
    #[test]
    #[ignore]
    fn blockchain() {
        let blockchain_info = block_on(localhost_rpc_client().blockchain(1u64, 10u64)).unwrap();
        assert_eq!(blockchain_info.block_metas.len(), 10);
    }

    /// `/commit` endpoint
    #[test]
    #[ignore]
    fn commit() {
        let height = 1u64;
        let commit_info = block_on(localhost_rpc_client().block(height)).unwrap();
        assert_eq!(commit_info.block_meta.header.height.value(), height);
    }

    /// `/genesis` endpoint
    #[test]
    #[ignore]
    fn genesis() {
        let genesis = block_on(localhost_rpc_client().genesis()).unwrap();
        assert_eq!(
            genesis.consensus_params.validator.pub_key_types[0].to_string(),
            "ed25519"
        );
    }

    /// `/net_info` endpoint integration test
    #[test]
    #[ignore]
    fn net_info() {
        let net_info = block_on(localhost_rpc_client().net_info()).unwrap();
        assert!(net_info.listening);
    }

    /// `/status` endpoint integration test
    #[test]
    #[ignore]
    fn status_integration() {
        let status = block_on(localhost_rpc_client().status()).unwrap();

        // For lack of better things to test
        assert_eq!(status.validator_info.voting_power.value(), 10);
    }
}
