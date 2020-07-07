//! Basic example of running the RPC server. This is a temporary show-case and should be removed
//! once integrated in the light node proper. To test the `/state` endpoint run:
//!
//! curl localhost:8888 -X POST -H 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "state", "id": 1}'

use tendermint_light_client::errors::Error;
use tendermint_light_client::supervisor::Handle;
use tendermint_light_client::types::LightBlock;

use tendermint_light_node::rpc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = MockHandle {};
    let server = rpc::Server::new(handle);

    Ok(rpc::run(server, "127.0.0.1:8888")?)
}

struct MockHandle;

impl Handle for MockHandle {
    fn latest_trusted(&self) -> Result<Option<LightBlock>, Error> {
        let block: LightBlock = serde_json::from_str(LIGHTBLOCK_JSON).unwrap();

        Ok(Some(block))
    }
}

const LIGHTBLOCK_JSON: &str = r#"
{
    "signed_header": {
            "header": {
                    "version": {
                            "block": "0",
                            "app": "0"
                    },
                    "chain_id": "test-chain-01",
                    "height": "1",
                    "time": "2019-11-02T15:04:00Z",
                    "last_block_id": {
                            "hash": "",
                            "parts": {
                                    "total": "0",
                                    "hash": ""
                            }
                    },
                    "last_commit_hash": "",
                    "data_hash": "",
                    "validators_hash": "ADAE23D9D908638F3866C11A39E31CE4399AE6DE8EC8EBBCB1916B90C46EDDE3",
                    "next_validators_hash": "ADAE23D9D908638F3866C11A39E31CE4399AE6DE8EC8EBBCB1916B90C46EDDE3",
                    "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
                    "app_hash": "6170705F68617368",
                    "last_results_hash": "",
                    "evidence_hash": "",
                    "proposer_address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33"
            },
            "commit": {
                    "height": "1",
                    "round": "1",
                    "block_id": {
                            "hash": "76B0FB738138A2C934300D7B23C280B65965D7427DA4D5414B41C75EBC4AD4C3",
                            "parts": {
                                    "total": "1",
                                    "hash": "073CE26981DF93820595E602CE63B810BC8F1003D6BB28DEDFF5B2F4F09811A1"
                            }
                    },
                    "signatures": [
                            {
                                    "block_id_flag": 2,
                                    "validator_address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                                    "timestamp": "2019-11-02T15:04:10Z",
                                    "signature": "NaNXQhv7SgBtcq+iHwItxlYUMGHP5MeFpTbyNsnLtzwM6P/EAAAexUH94+osvRDoiahUOoQrRlTiZrYGfahWBw=="
                            },
                            {
                                    "block_id_flag": 2,
                                    "validator_address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                                    "timestamp": "2019-11-02T15:04:10Z",
                                    "signature": "tw0csJ1L1vkBG/71BMjrFEcA6VWjOx29WMwkg1cmDn82XBjRFz+HJu7amGoIj6WLL2p26pO25yQR49crsYQ+AA=="
                            }
                    ]
            }
    },
    "validator_set": {
            "validators": [
                    {
                            "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                            },
                            "voting_power": "50",
                            "proposer_priority": "-50"
                    },
                    {
                            "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                            },
                            "voting_power": "50",
                            "proposer_priority": "50"
                    }
            ],
            "proposer": {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                            "type": "tendermint/PubKeyEd25519",
                            "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "50",
                    "proposer_priority": "-50"
            }
    },
    "next_validator_set": {
            "validators": [
                    {
                            "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                            },
                            "voting_power": "50",
                            "proposer_priority": "0"
                    },
                    {
                            "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                            },
                            "voting_power": "50",
                            "proposer_priority": "0"
                    }
            ],
            "proposer": {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                            "type": "tendermint/PubKeyEd25519",
                            "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "50",
                    "proposer_priority": "0" }
    },
    "provider": "9D61B19DEFFD5A60BA844AF492EC2CC44449C569"
}
"#;
