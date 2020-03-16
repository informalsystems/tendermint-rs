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
    use tendermint::rpc::Client;

    /// Get the address of the local node
    pub fn localhost_rpc_client() -> Client {
        Client::new("tcp://127.0.0.1:26657".parse().unwrap())
    }

    /// `/health` endpoint
    #[tokio::test]
    #[ignore]
    async fn health() {
        let result = localhost_rpc_client().health().await;

        assert!(result.is_ok(), "health check failed");
    }

    /// `/abci_info` endpoint
    #[tokio::test]
    #[ignore]
    async fn abci_info() {
        let abci_info = localhost_rpc_client().abci_info().await.unwrap();

        assert_eq!(&abci_info.data, "GaiaApp");
    }

    /// `/abci_query` endpoint
    #[tokio::test]
    #[ignore]
    async fn abci_query() {
        let key = "unpopulated_key".parse().unwrap();
        let abci_query = localhost_rpc_client()
            .abci_query(Some(key), vec![], None, false)
            .await
            .unwrap();

        assert_eq!(abci_query.key.as_ref().unwrap(), &Vec::<u8>::new());
        assert_eq!(abci_query.value.as_ref(), None);
    }

    /// `/block` endpoint
    #[tokio::test]
    #[ignore]
    async fn block() {
        let height = 1u64;
        let block_info = localhost_rpc_client().block(height).await.unwrap();

        assert_eq!(block_info.block.header.height.value(), height);
    }

    /// `/block_results` endpoint
    #[tokio::test]
    #[ignore]
    async fn block_results() {
        let height = 1u64;
        let block_results = localhost_rpc_client().block_results(height).await.unwrap();

        assert_eq!(block_results.height.value(), height);
    }

    /// `/blockchain` endpoint
    #[tokio::test]
    #[ignore]
    async fn blockchain() {
        let blockchain_info = localhost_rpc_client()
            .blockchain(1u64, 10u64)
            .await
            .unwrap();

        assert_eq!(blockchain_info.block_metas.len(), 10);
    }

    /// `/commit` endpoint
    #[tokio::test]
    #[ignore]
    async fn commit() {
        let height = 1u64;
        let commit_info = localhost_rpc_client().block(height).await.unwrap();

        assert_eq!(commit_info.block.header.height.value(), height);
    }

    /// `/genesis` endpoint
    #[tokio::test]
    #[ignore]
    async fn genesis() {
        let genesis = localhost_rpc_client().genesis().await.unwrap();

        assert_eq!(
            genesis.consensus_params.validator.pub_key_types[0].to_string(),
            "ed25519"
        );
    }

    /// `/net_info` endpoint integration test
    #[tokio::test]
    #[ignore]
    async fn net_info() {
        let net_info = localhost_rpc_client().net_info().await.unwrap();

        assert!(net_info.listening);
    }

    /// `/status` endpoint integration test
    #[tokio::test]
    #[ignore]
    async fn status_integration() {
        let status = localhost_rpc_client().status().await.unwrap();

        // For lack of better things to test
        assert_eq!(status.validator_info.voting_power.value(), 10);
    }
}
