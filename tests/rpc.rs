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
    fn block() {
        let block_json = read_json_fixture("block");
        let block_response = endpoint::block::Response::from_json(&block_json).unwrap();

        let tendermint::Block {
            header,
            data,
            evidence,
            last_commit,
        } = block_response.block;

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
        let commit_json = read_json_fixture("commit");
        let commit_response = endpoint::commit::Response::from_json(&commit_json).unwrap();

        println!("commit_response: {:?}", commit_response);
    }

    #[test]
    fn genesis() {
        let genesis_json = read_json_fixture("genesis");
        let genesis_response = endpoint::genesis::Response::from_json(&genesis_json).unwrap();

        let tendermint::Genesis {
            chain_id,
            consensus_params,
            ..
        } = genesis_response.genesis;

        assert_eq!(chain_id.as_str(), "cosmoshub-1");
        assert_eq!(consensus_params.block_size.max_bytes, 150000);
    }

    #[test]
    fn net_info() {
        let net_info_json = read_json_fixture("net_info");
        let net_info_response = endpoint::net_info::Response::from_json(&net_info_json).unwrap();

        assert_eq!(net_info_response.n_peers, 2);
        assert_eq!(
            net_info_response.peers[0].node_info.network.as_str(),
            "cosmoshub-1"
        );
    }

    #[test]
    fn status() {
        let status_json = read_json_fixture("status");
        let status_response = endpoint::status::Response::from_json(&status_json).unwrap();

        assert_eq!(status_response.node_info.network.as_str(), "cosmoshub-1");
        assert_eq!(
            status_response.sync_info.latest_block_height.value(),
            410744
        );
        assert_eq!(status_response.validator_info.voting_power.value(), 0);
    }

    #[test]
    fn validators() {
        let validators_json = read_json_fixture("validators");
        let validators_response =
            endpoint::validators::Response::from_json(&validators_json).unwrap();

        println!("validators: {:?}", validators_response);
    }
}
