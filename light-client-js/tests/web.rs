//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use tendermint::Time;
use tendermint_light_client_js::{verify, Error, JsOptions};
use tendermint_light_client_verifier::{types::LightBlock, Verdict};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const UNTRUSTED_BLOCK: &str = r#"{
    "signed_header": {
        "header": {
            "version": {
                "block": "11",
                "app": "0"
            },
            "chain_id": "test-chain",
            "height": "4",
            "time": "1970-01-01T00:00:04Z",
            "last_block_id": null,
            "last_commit_hash": null,
            "data_hash": null,
            "validators_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "next_validators_hash": "C8CFFADA9808F685C4111693E1ADFDDBBEE9B9493493BEF805419F143C5B0D0A",
            "consensus_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "app_hash": "",
            "last_results_hash": null,
            "evidence_hash": null,
            "proposer_address": "6AE5C701F508EB5B63343858E068C5843F28105F"
        },
        "commit": {
            "height": "4",
            "round": 1,
            "block_id": {
                "hash": "D0E7B0C678E290DA835BB26EE826472D66B6A306801E5FE0803C5320C554610A",
                "part_set_header": {
                    "total": 1,
                    "hash": "D0E7B0C678E290DA835BB26EE826472D66B6A306801E5FE0803C5320C554610A"
                }
            },
            "signatures": [
                {
                    "block_id_flag": 2,
                    "validator_address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                    "timestamp": "1970-01-01T00:00:04Z",
                    "signature": "lTGBsjVI6YwIRcxQ6Lct4Q+xrtJc9h3648c42uWe4MpSgy4rUI5g71AEpG90Tbn0PRizjKgCPhokPpQoQLiqAg=="
                }
            ]
        }
    },
    "validator_set": {
        "total_voting_power": "0",
        "validators": [
            {
                "address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "GQEC/HB4sDBAVhHtUzyv4yct9ZGnudaP209QQBSTfSQ="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "next_validator_set": {
        "total_voting_power": "0",
        "validators": [
            {
                "address": "C479DB6F37AB9757035CFBE10B687E27668EE7DF",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "3wf60CidQcsIO7TksXzEZsJefMUFF73k6nP1YeEo9to="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "provider": "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE"
}"#;

const TRUSTED_BLOCK: &str = r#"{
    "signed_header": {
        "header": {
            "version": {
                "block": "11",
                "app": "0"
            },
            "chain_id": "test-chain",
            "height": "3",
            "time": "1970-01-01T00:00:03Z",
            "last_block_id": null,
            "last_commit_hash": null,
            "data_hash": null,
            "validators_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "next_validators_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "consensus_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "app_hash": "",
            "last_results_hash": null,
            "evidence_hash": null,
            "proposer_address": "6AE5C701F508EB5B63343858E068C5843F28105F"
        },
        "commit": {
            "height": "3",
            "round": 1,
            "block_id": {
                "hash": "AAB1B09D5FADAAE7CDF3451961A63F810DB73BF3214A7B74DBA36C52EDF1A793",
                "part_set_header": {
                    "total": 1,
                    "hash": "AAB1B09D5FADAAE7CDF3451961A63F810DB73BF3214A7B74DBA36C52EDF1A793"
                }
            },
            "signatures": [
                {
                    "block_id_flag": 2,
                    "validator_address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                    "timestamp": "1970-01-01T00:00:03Z",
                    "signature": "xn0eSsHYIsqUbmfAiJq1R0hqZbfuIjs5Na1c88EC1iPTuQAesKg9I7nXG4pk8d6U5fU4GysNLk5I4f7aoefOBA=="
                }
            ]
        }
    },
    "validator_set": {
        "total_voting_power": "0",
        "validators": [
            {
                "address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "GQEC/HB4sDBAVhHtUzyv4yct9ZGnudaP209QQBSTfSQ="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "next_validator_set": {
        "total_voting_power": "0",
        "validators": [
            {
                "address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "GQEC/HB4sDBAVhHtUzyv4yct9ZGnudaP209QQBSTfSQ="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "provider": "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE"
}"#;

#[wasm_bindgen_test]
fn successful_verification() {
    let (untrusted_block, trusted_block) = test_blocks();
    let options = test_options();
    // Choose a "now" value within the trusting period
    let now =
        serde_wasm_bindgen::to_value(&Time::parse_from_rfc3339("1970-01-07T00:00:00Z").unwrap())
            .unwrap();
    let js_result = verify(&untrusted_block, &trusted_block, &options, &now);
    console_log!("js_result = {:?}", js_result);
    let verdict = serde_wasm_bindgen::from_value::<Result<Verdict, Error>>(js_result)
        .unwrap()
        .unwrap();
    assert_eq!(verdict, Verdict::Success);
}

#[wasm_bindgen_test]
fn failed_verification_outside_trusting_period() {
    let (untrusted_block, trusted_block) = test_blocks();
    let options = test_options();
    // Choose a "now" value outside the trusting period
    let now =
        serde_wasm_bindgen::to_value(&Time::parse_from_rfc3339("1970-01-16T00:00:00Z").unwrap())
            .unwrap();
    let js_result = verify(&untrusted_block, &trusted_block, &options, &now);
    console_log!("js_result = {:?}", js_result);
    // The result is Ok because we successfully obtained a verdict, even if the
    // verdict isn't Verdict::Success.
    let verdict = serde_wasm_bindgen::from_value::<Result<Verdict, Error>>(js_result)
        .unwrap()
        .unwrap();
    match verdict {
        Verdict::Success | Verdict::NotEnoughTrust(_) => panic!("unexpected verdict"),
        _ => {},
    }
}

fn test_blocks() -> (JsValue, JsValue) {
    let untrusted_block =
        serde_wasm_bindgen::to_value(&serde_json::from_str::<LightBlock>(UNTRUSTED_BLOCK).unwrap())
            .unwrap();
    let trusted_block =
        serde_wasm_bindgen::to_value(&serde_json::from_str::<LightBlock>(TRUSTED_BLOCK).unwrap())
            .unwrap();
    (untrusted_block, trusted_block)
}

fn test_options() -> JsValue {
    serde_wasm_bindgen::to_value(&JsOptions {
        trust_threshold: (1, 3),
        trusting_period: 1209600, // 2 weeks
        clock_drift: 5,           // 5 seconds
    })
    .unwrap()
}
