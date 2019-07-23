//! Tendermint RPC tests

#[cfg(feature = "rpc")]
mod endpoints {
    use std::{fs, path::PathBuf};
    use tendermint::rpc::{self, endpoint, Response};

    const EXAMPLE_APP: &str = "GaiaApp";
    const EXAMPLE_CHAIN: &str = "cosmoshub-1";

    fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/rpc/").join(name.to_owned() + ".json"))
            .unwrap()
    }

    #[test]
    fn abci_info() {
        let response = endpoint::abci_info::Response::from_json(&read_json_fixture("abci_info"))
            .unwrap()
            .response;

        assert_eq!(response.data.as_str(), EXAMPLE_APP);
        assert_eq!(response.last_block_height.value(), 488120);
    }

    #[test]
    fn abci_query() {
        let response = endpoint::abci_query::Response::from_json(&read_json_fixture("abci_query"))
            .unwrap()
            .response;

        assert_eq!(response.height.value(), 1);
    }

    #[test]
    fn block() {
        let response = endpoint::block::Response::from_json(&read_json_fixture("block")).unwrap();

        let tendermint::Block {
            header,
            data,
            evidence,
            last_commit,
        } = response.block;

        assert_eq!(header.version.block, 10);
        assert_eq!(header.chain_id.as_str(), EXAMPLE_CHAIN);
        assert_eq!(header.height.value(), 15);
        assert_eq!(header.num_txs, 2);

        assert_eq!(data.iter().len(), 2);
        assert_eq!(evidence.iter().len(), 0);
        assert_eq!(last_commit.precommits.len(), 65);
    }

    #[test]
    fn block_results() {
        let response =
            endpoint::block_results::Response::from_json(&read_json_fixture("block_results"))
                .unwrap();
        assert_eq!(response.height.value(), 1814);

        let tendermint::abci::Responses {
            deliver_tx,
            begin_block: _,
            end_block,
        } = response.results;

        let log_json = &deliver_tx[0].log.as_ref().unwrap().parse_json().unwrap();
        let log_json_value = &log_json.as_array().as_ref().unwrap()[0];

        assert_eq!(log_json_value["msg_index"].as_str().unwrap(), "0");
        assert_eq!(log_json_value["success"].as_bool().unwrap(), true);

        assert_eq!(deliver_tx[0].gas_wanted.value(), 200000);
        assert_eq!(deliver_tx[0].gas_used.value(), 105662);

        let tag = deliver_tx[0]
            .tags
            .iter()
            .find(|t| t.key.as_ref().eq("ZGVzdGluYXRpb24tdmFsaWRhdG9y"))
            .unwrap();

        assert_eq!(
            tag.value.as_ref(),
            "Y29zbW9zdmFsb3BlcjFlaDVtd3UwNDRnZDVudGtrYzJ4Z2ZnODI0N21nYzU2Zno0c2RnMw=="
        );

        let validator_update = &end_block.as_ref().unwrap().validator_updates[0];
        assert_eq!(validator_update.power.value(), 1233243);
    }

    #[test]
    fn blockchain() {
        let response =
            endpoint::blockchain::Response::from_json(&read_json_fixture("blockchain")).unwrap();

        assert_eq!(response.last_height.value(), 488556);
        assert_eq!(response.block_metas.len(), 10);

        let block_meta = &response.block_metas[0];
        assert_eq!(block_meta.header.chain_id.as_str(), EXAMPLE_CHAIN)
    }

    #[test]
    fn broadcast_tx_async() {
        let response = endpoint::broadcast::tx_async::Response::from_json(&read_json_fixture(
            "broadcast_tx_async",
        ))
        .unwrap();

        assert_eq!(
            &response.hash.to_string(),
            "E39AAB7A537ABAA237831742DCE1117F187C3C52"
        );
    }

    #[test]
    fn broadcast_tx_sync() {
        let response = endpoint::broadcast::tx_sync::Response::from_json(&read_json_fixture(
            "broadcast_tx_sync",
        ))
        .unwrap();

        assert_eq!(
            &response.hash.to_string(),
            "0D33F2F03A5234F38706E43004489E061AC40A2E"
        );
    }

    #[test]
    fn broadcast_tx_commit() {
        let response = endpoint::broadcast::tx_commit::Response::from_json(&read_json_fixture(
            "broadcast_tx_commit",
        ))
        .unwrap();

        assert_eq!(
            &response.hash.to_string(),
            "75CA0F856A4DA078FC4911580360E70CEFB2EBEE"
        );
    }

    #[test]
    fn commit() {
        let response = endpoint::commit::Response::from_json(&read_json_fixture("commit")).unwrap();
        let header = response.signed_header.header;
        assert_eq!(header.chain_id.as_ref(), EXAMPLE_CHAIN);
    }

    #[test]
    fn genesis() {
        let response =
            endpoint::genesis::Response::from_json(&read_json_fixture("genesis")).unwrap();

        let tendermint::Genesis {
            chain_id,
            consensus_params,
            ..
        } = response.genesis;

        assert_eq!(chain_id.as_str(), EXAMPLE_CHAIN);
        assert_eq!(consensus_params.block.max_bytes, 200000);
    }

    #[test]
    fn health() {
        endpoint::health::Response::from_json(&read_json_fixture("health")).unwrap();
    }

    #[test]
    fn net_info() {
        let response =
            endpoint::net_info::Response::from_json(&read_json_fixture("net_info")).unwrap();

        assert_eq!(response.n_peers, 2);
        assert_eq!(response.peers[0].node_info.network.as_str(), EXAMPLE_CHAIN);
    }

    #[test]
    fn status() {
        let response = endpoint::status::Response::from_json(&read_json_fixture("status")).unwrap();

        assert_eq!(response.node_info.network.as_str(), EXAMPLE_CHAIN);
        assert_eq!(response.sync_info.latest_block_height.value(), 410744);
        assert_eq!(response.validator_info.voting_power.value(), 0);
    }

    #[test]
    fn validators() {
        let response =
            endpoint::validators::Response::from_json(&read_json_fixture("validators")).unwrap();

        assert_eq!(response.block_height.value(), 42);

        let validators = response.validators;
        assert_eq!(validators.len(), 65);
    }

    #[test]
    fn jsonrpc_error() {
        let result = endpoint::blockchain::Response::from_json(&read_json_fixture("error"));

        if let Err(err) = result {
            assert_eq!(err.code(), rpc::error::Code::InternalError);
            assert_eq!(err.message(), "Internal error");
            assert_eq!(
                err.data().unwrap(),
                "min height 321 can't be greater than max height 123"
            );
        } else {
            panic!("expected error, got {:?}", result)
        }
    }
}
