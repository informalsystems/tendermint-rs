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
    use tendermint_rpc::endpoint::abci_info::AbciInfo;
    use tendermint_rpc::event::{Event, EventData};
    use tendermint_rpc::query::EventType;
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
        assert_eq!(abci_info.data.is_empty(), false);
    }

    /// `/abci_info` endpoint JSON serialization test
    #[tokio::test]
    #[ignore]
    async fn fake_abci_info() {
        use serde::{Deserialize, Serialize};
        use std::convert::TryInto;
        use tendermint::Error;
        use tendermint_proto::abci::ResponseInfo;
        use tendermint_proto::DomainType;

        // A domain type that has a new type of JSON serialization:
        // * The default Serialize and Deserialize traits are derived
        // * serde is told to use a Raw type for serialization called "ResponseInfo"
        // * The TryFrom/Into traits are implemented for this Raw type, because of protobuf
        //   encoding.
        // * Custom serializers were completely removed from here and moved into tendermint-proto
        #[derive(Clone, Debug, Deserialize, Serialize, Default)]
        #[serde(default, try_from = "ResponseInfo", into = "ResponseInfo")]
        pub struct FakeAbciInfo {
            /// Name of the application
            pub data: String,

            /// Version
            pub version: String,

            /// App version
            //Previously: #[serde(with = "serializers::from_str")]
            //Now: moved to tendermint-proto
            pub app_version: u64,

            /// Last block height
            pub last_block_height: tendermint::block::Height,

            /// Last app hash for the block
            //Previously: #[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
            //Now: moved to tendermint-proto
            pub last_block_app_hash: Vec<u8>,
        }

        // The below DomainType, TryFrom and From traits are already implemented for all domain
        // types.
        impl DomainType<ResponseInfo> for FakeAbciInfo {}

        impl TryFrom<ResponseInfo> for FakeAbciInfo {
            type Error = Error;

            fn try_from(value: ResponseInfo) -> Result<Self, Self::Error> {
                Ok(FakeAbciInfo {
                    data: value.data,
                    version: value.version,
                    app_version: value.app_version,
                    last_block_height: value.last_block_height.try_into()?,
                    last_block_app_hash: value.last_block_app_hash,
                })
            }
        }

        impl From<FakeAbciInfo> for ResponseInfo {
            fn from(value: FakeAbciInfo) -> Self {
                ResponseInfo {
                    data: value.data,
                    version: value.version,
                    app_version: value.app_version,
                    last_block_height: value.last_block_height.into(),
                    last_block_app_hash: value.last_block_app_hash,
                }
            }
        }

        // AbciInfo JSON string copied from a /abci_info request to tendermint Go v0.34.0-rc5
        let abci_info_json = r#"
{
    "data": "{\"size\":40}",
    "version": "0.17.0",
    "app_version": "1",
    "last_block_height": "2653",
    "last_block_app_hash": "UAAAAAAAAAA="
}
"#;
        // Old encoding
        let abci_info_old: AbciInfo = serde_json::from_str(abci_info_json).unwrap();

        // New encoding
        let abci_info_new: FakeAbciInfo = serde_json::from_str(abci_info_json).unwrap();

        assert_eq!(abci_info_old.data, abci_info_new.data);
        assert_eq!(abci_info_old.version, abci_info_new.version);
        assert_eq!(abci_info_old.app_version, abci_info_new.app_version);
        assert_eq!(
            abci_info_old.last_block_height,
            abci_info_new.last_block_height
        );
        assert_eq!(
            abci_info_old.last_block_app_hash,
            abci_info_new.last_block_app_hash
        );
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
            block_info
                .block
                .header
                .data_hash
                .unwrap_or_default()
                .as_bytes()
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
            .blockchain(Height::from(1u32), Height::try_from(max_height).unwrap())
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
        let (mut client, driver) = WebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
            .await
            .unwrap();
        let driver_handle = tokio::spawn(async move { driver.run().await });
        let mut subs = client.subscribe(EventType::NewBlock.into()).await.unwrap();
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

        client.close().await.unwrap();
        let _ = driver_handle.await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn transaction_subscription() {
        // We run these sequentially wrapped within a single test to ensure
        // that Tokio doesn't execute them simultaneously. If they are executed
        // simultaneously, their submitted transactions interfere with each
        // other and one of them will (incorrectly) fail.
        simple_transaction_subscription().await;
        concurrent_subscriptions().await;
    }

    async fn simple_transaction_subscription() {
        let rpc_client = HttpClient::new("tcp://127.0.0.1:26657".parse().unwrap()).unwrap();
        let (mut subs_client, driver) =
            WebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
                .await
                .unwrap();
        let driver_handle = tokio::spawn(async move { driver.run().await });
        let mut subs = subs_client.subscribe(EventType::Tx.into()).await.unwrap();
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

        subs_client.close().await.unwrap();
        let _ = driver_handle.await.unwrap();
    }

    async fn concurrent_subscriptions() {
        let rpc_client = HttpClient::new("tcp://127.0.0.1:26657".parse().unwrap()).unwrap();
        let (mut subs_client, driver) =
            WebSocketClient::new("tcp://127.0.0.1:26657".parse().unwrap())
                .await
                .unwrap();
        let driver_handle = tokio::spawn(async move { driver.run().await });
        let new_block_subs = subs_client
            .subscribe(EventType::NewBlock.into())
            .await
            .unwrap();
        let tx_subs = subs_client.subscribe(EventType::Tx.into()).await.unwrap();

        // We use Id::uuid_v4() here as a quick hack to generate a random value.
        let mut expected_tx_values = (0..10_u32)
            .map(|_| Id::uuid_v4().to_string())
            .collect::<Vec<String>>();
        let broadcast_tx_values = expected_tx_values.clone();
        let mut expected_new_blocks = 5_i32;

        tokio::spawn(async move {
            for (tx_count, val) in broadcast_tx_values.into_iter().enumerate() {
                let tx = format!("tx{}={}", tx_count, val);
                rpc_client
                    .broadcast_tx_async(Transaction::new(tx.as_bytes()))
                    .await
                    .unwrap();
                tokio::time::delay_for(Duration::from_millis(100)).await;
            }
        });

        let mut combined_subs = futures::stream::select_all(vec![new_block_subs, tx_subs]);

        println!(
            "Attempting to receive {} transactions and {} new blocks",
            expected_tx_values.len(),
            expected_new_blocks
        );

        while expected_new_blocks > 0 && !expected_tx_values.is_empty() {
            let mut timeout = tokio::time::delay_for(Duration::from_secs(3));
            tokio::select! {
                Some(res) = combined_subs.next() => {
                    let ev: Event = res.unwrap();
                    println!("Got event: {:?}", ev);
                    match ev.data {
                        EventData::NewBlock { .. } => {
                            println!("Got new block event");
                            expected_new_blocks -= 1;
                        },
                        EventData::Tx { .. } => {
                            println!("Got new transaction event");
                            let _ = expected_tx_values.pop();
                        },
                        _ => panic!("Unexpected event received: {:?}", ev),
                    }
                },
                _ = &mut timeout => panic!("Timed out waiting for an event"),
            }
        }

        subs_client.close().await.unwrap();
        let _ = driver_handle.await.unwrap();
    }
}
