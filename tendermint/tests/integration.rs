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
    use futures::executor::block_on;
    use std::cmp::min;

    use tendermint_rpc::{event_listener, Client};

    use tendermint::abci::Code;
    use tendermint::abci::Log;

    /// Get the address of the local node
    pub fn localhost_rpc_client() -> Client {
        Client::new("tcp://127.0.0.1:26657".parse().unwrap())
    }

    /// `/health` endpoint
    #[test]
    #[ignore]
    fn health() {
        let result = block_on(localhost_rpc_client().health());

        assert!(result.is_ok(), "health check failed");
    }

    /// `/abci_info` endpoint
    #[test]
    #[ignore]
    fn abci_info() {
        let abci_info = block_on(localhost_rpc_client().abci_info()).unwrap();

        assert_eq!(abci_info.app_version, 1u64);
        // the kvstore app's reply will contain "{\"size\":0}" as data right from the start
        assert_eq!(&abci_info.data, "{\"size\":0}");
        assert_eq!(abci_info.data.is_empty(), false);
        assert_eq!(abci_info.last_block_app_hash[0], 65);
    }

    /// `/abci_query` endpoint
    #[test]
    #[ignore]
    fn abci_query() {
        let key = "unpopulated_key".parse().unwrap();
        let abci_query =
            block_on(localhost_rpc_client().abci_query(Some(key), vec![], None, false)).unwrap();

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
    #[test]
    #[ignore]
    fn block() {
        let height = 1u64;
        let block_info = block_on(localhost_rpc_client().block(height)).unwrap();

        assert!(block_info.block.last_commit.is_none());
        assert_eq!(block_info.block.header.height.value(), height);
    }

    /// `/block_results` endpoint
    #[test]
    #[ignore]
    fn block_results() {
        let height = 1u64;
        let block_results = block_on(localhost_rpc_client().block_results(height)).unwrap();

        assert_eq!(block_results.height.value(), height);
        assert!(block_results.txs_results.is_none());
    }

    /// `/blockchain` endpoint
    #[test]
    #[ignore]
    fn blockchain() {
        let max_height = 10u64;
        let blockchain_info =
            block_on(localhost_rpc_client().blockchain(1u64, max_height)).unwrap();

        assert_eq!(
            blockchain_info.block_metas.len() as u64,
            min(max_height, blockchain_info.last_height.value())
        );
    }

    /// `/commit` endpoint
    #[test]
    #[ignore]
    fn commit() {
        let height = 1u64;
        let commit_info = block_on(localhost_rpc_client().commit(height)).unwrap();

        assert_eq!(commit_info.signed_header.header.height.value(), height);
        assert_eq!(commit_info.canonical, true);
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

    #[test]
    #[ignore]
    fn event_subscription() {
        let mut client = block_on(event_listener::EventListener::connect(
            "tcp://127.0.0.1:26657".parse().unwrap(),
        ))
        .unwrap();
        block_on(client.subscribe(event_listener::EventSubscription::BlockSubscription)).unwrap();
        // client.subscribe("tm.event='NewBlock'".to_owned()).await.unwrap();

        // Loop here is helpful when debugging parsing of JSON events
        // loop{
        let maybe_result_event = block_on(client.get_event()).unwrap();
        dbg!(&maybe_result_event);
        // }
        let result_event = maybe_result_event.expect("unexpected msg read");
        match result_event.data {
            event_listener::TMEventData::EventDataNewBlock(nb) => {
                dbg!("got EventDataNewBlock: {:?}", nb);
            }
            event_listener::TMEventData::EventDataTx(tx) => {
                dbg!("got EventDataTx: {:?}", tx);
            }
            event_listener::TMEventData::GenericJSONEvent(v) => {
                panic!("got a GenericJSONEvent: {:?}", v);
            }
        }
    }
}
