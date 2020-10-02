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
    use std::cmp::min;

    use tendermint_rpc::{Client, HttpClient, Id, SubscriptionClient, WebSocketClient};

    use futures::StreamExt;
    use std::convert::TryFrom;
    use subtle_encoding::base64;
    use tendermint::abci::Log;
    use tendermint::abci::{Code, Transaction};
    use tendermint::block::Height;
    use tendermint::merkle::simple_hash_from_byte_vectors;
    use tendermint_rpc::event::EventData;
    use tokio::time::Duration;

    /// Get the address of the local node
    pub fn localhost_rpc_client() -> HttpClient {
        HttpClient::new("tcp://127.0.0.1:26657".parse().unwrap()).unwrap()
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

        assert_eq!(abci_info.app_version, 1u64);
        // the kvstore app's reply will contain "{\"size\":0}" as data right from the start
        assert_eq!(&abci_info.data, "{\"size\":0}");
        assert_eq!(abci_info.data.is_empty(), false);
        assert_eq!(abci_info.last_block_app_hash[0], 65);
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

        assert_eq!(abci_query.code, Code::Ok);
        assert_eq!(abci_query.log, Log::from("does not exist"));
        assert_eq!(abci_query.info, String::new());
        assert_eq!(abci_query.index, 0);
        assert_eq!(&abci_query.key, &Vec::<u8>::new());
        assert!(&abci_query.key.is_empty());
        assert_eq!(abci_query.value, Vec::<u8>::new());
        assert!(abci_query.proof.is_none());
        assert!(abci_query.height.value() > 0);
        assert_eq!(abci_query.codespace, String::new());
    }

    /// `/block` endpoint
    #[tokio::test]
    #[ignore]
    async fn block() {
        let height = 1u64;
        let block_info = localhost_rpc_client()
            .block(Height::try_from(height).unwrap())
            .await
            .unwrap();

        assert!(block_info.block.last_commit.is_none());
        assert_eq!(block_info.block.header.height.value(), height);

        // Check for empty merkle root.
        // See: https://github.com/informalsystems/tendermint-rs/issues/562
        let computed_data_hash = simple_hash_from_byte_vectors(
            block_info
                .block
                .data
                .iter()
                .map(|t| t.to_owned().into_vec())
                .collect(),
        );
        assert_eq!(
            computed_data_hash,
            block_info.block.header.data_hash.unwrap().as_bytes()
        );
    }

    /// `/block_results` endpoint
    #[tokio::test]
    #[ignore]
    async fn block_results() {
        let height = 1u64;
        let block_results = localhost_rpc_client()
            .block_results(Height::try_from(height).unwrap())
            .await
            .unwrap();

        assert_eq!(block_results.height.value(), height);
        assert!(block_results.txs_results.is_none());
    }

    /// `/blockchain` endpoint
    #[tokio::test]
    #[ignore]
    async fn blockchain() {
        let max_height = 10u64;
        let blockchain_info = localhost_rpc_client()
            .blockchain(
                Height::try_from(1u64).unwrap(),
                Height::try_from(max_height).unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            blockchain_info.block_metas.len() as u64,
            min(max_height, blockchain_info.last_height.value())
        );
    }

    /// `/commit` endpoint
    #[tokio::test]
    #[ignore]
    async fn commit() {
        let height = 1u64;
        let commit_info = localhost_rpc_client()
            .commit(Height::try_from(height).unwrap())
            .await
            .unwrap();

        assert_eq!(commit_info.signed_header.header.height.value(), height);
        assert_eq!(commit_info.canonical, true);
        assert_eq!(
            commit_info.signed_header.header.hash(),
            commit_info.signed_header.commit.block_id.hash
        );
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

    #[tokio::test]
    #[ignore]
    async fn subscription_interface() {
        let mut client = WebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
            .await
            .unwrap();
        let mut subs = client
            .subscribe("tm.event='NewBlock'".to_string())
            .await
            .unwrap();
        let mut ev_count = 5_i32;

        println!("Attempting to grab {} new blocks", ev_count);
        while let Some(res) = subs.next().await {
            let ev = res.unwrap();
            println!("Got event: {:?}", ev);
            ev_count -= 1;
            if ev_count < 0 {
                break;
            }
        }

        subs.terminate().await.unwrap();
        client.close().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn transaction_subscription() {
        let rpc_client = HttpClient::new("tcp://127.0.0.1:26657".parse().unwrap()).unwrap();
        let mut subs_client = WebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
            .await
            .unwrap();
        let mut subs = subs_client
            .subscribe("tm.event='Tx'".to_string())
            .await
            .unwrap();
        // We use Id::uuid_v4() here as a quick hack to generate a random value.
        let mut expected_tx_values = (0..10_u32)
            .map(|_| Id::uuid_v4().to_string())
            .collect::<Vec<String>>();
        let broadcast_tx_values = expected_tx_values.clone();

        tokio::spawn(async move {
            for (tx_count, val) in broadcast_tx_values.into_iter().enumerate() {
                let tx = format!("tx{}={}", tx_count, val);
                rpc_client
                    .broadcast_tx_async(Transaction::new(tx.as_bytes()))
                    .await
                    .unwrap();
            }
        });

        println!(
            "Attempting to grab {} transaction events",
            expected_tx_values.len()
        );
        let mut cur_tx_id = 0_u32;

        while !expected_tx_values.is_empty() {
            let mut delay = tokio::time::delay_for(Duration::from_secs(3));
            tokio::select! {
                Some(res) = subs.next() => {
                    let ev = res.unwrap();
                    //println!("Got event: {:?}", ev);
                    let next_val = expected_tx_values.remove(0);
                    match ev.data {
                        EventData::Tx { tx_result } => match base64::decode(tx_result.tx) {
                            Ok(decoded_tx) => match String::from_utf8(decoded_tx) {
                                Ok(decoded_tx_str) => {
                                    let decoded_tx_split = decoded_tx_str
                                        .split('=')
                                        .map(|s| s.to_string())
                                        .collect::<Vec<String>>();
                                    assert_eq!(2, decoded_tx_split.len());

                                    let key = decoded_tx_split.get(0).unwrap();
                                    let val = decoded_tx_split.get(1).unwrap();
                                    println!("Got tx: {}={}", key, val);
                                    assert_eq!(format!("tx{}", cur_tx_id), *key);
                                    assert_eq!(next_val, *val);
                                }
                                Err(e) => panic!("Failed to convert decoded tx to string: {}", e),
                            },
                            Err(e) => panic!("Failed to base64 decode tx from event: {}", e),
                        },
                        _ => panic!("Unexpected event type: {:?}", ev),
                    }
                    cur_tx_id += 1;
                },
                _ = &mut delay => panic!("Timed out waiting for an event"),
            }
        }

        subs.terminate().await.unwrap();
        subs_client.close().await.unwrap();
    }
}
