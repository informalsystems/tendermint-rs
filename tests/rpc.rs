//! Tendermint RPC tests

#[cfg(feature = "rpc")]
mod endpoints {
    use std::{fs, path::PathBuf};
    use tendermint::rpc::{endpoint, Response};

    fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/rpc/").join(name.to_owned() + ".json"))
            .unwrap()
    }

    #[test]
    fn abci_info() {
        let response = endpoint::abci_info::Response::from_json(&read_json_fixture("abci_info"))
            .unwrap()
            .response;

        assert_eq!(response.data.as_str(), "GaiaApp");
        assert_eq!(response.last_block_height.value(), 488120);
    }

    #[test]
    fn block() {
        let block = endpoint::block::Response::from_json(&read_json_fixture("block")).unwrap();

        let tendermint::Block {
            header,
            data,
            evidence,
            last_commit,
        } = block.block;

        assert_eq!(header.version.block, 10);
        assert_eq!(header.chain_id.as_str(), "cosmoshub-1");
        assert_eq!(header.height.value(), 15);
        assert_eq!(header.num_txs, 2);

        assert_eq!(data.iter().len(), 2);
        assert_eq!(evidence.iter().len(), 0);
        assert_eq!(last_commit.precommits.len(), 65);
    }

    #[test]
    fn commit() {
        let response = endpoint::commit::Response::from_json(&read_json_fixture("commit")).unwrap();
        let header = response.signed_header.header;

        assert_eq!(header.chain_id.as_ref(), "cosmoshub-1");
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

        assert_eq!(chain_id.as_str(), "cosmoshub-1");
        assert_eq!(consensus_params.block_size.max_bytes, 150000);
    }

    #[test]
    fn net_info() {
        let response =
            endpoint::net_info::Response::from_json(&read_json_fixture("net_info")).unwrap();

        assert_eq!(response.n_peers, 2);
        assert_eq!(response.peers[0].node_info.network.as_str(), "cosmoshub-1");
    }

    #[test]
    fn status() {
        let response = endpoint::status::Response::from_json(&read_json_fixture("status")).unwrap();

        assert_eq!(response.node_info.network.as_str(), "cosmoshub-1");
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
}
