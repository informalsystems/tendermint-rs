import * as monaco from 'monaco-editor';
import * as LightClient from 'tendermint-light-client-js';

let untrustedBlockEditor = monaco.editor.create(document.getElementById("untrusted-block-editor"), {
    value: JSON.stringify({
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
    }, null, 2),
    language: 'json',
    minimap: { enabled: false }
});

let trustedBlockEditor = monaco.editor.create(document.getElementById("trusted-block-editor"), {
    value: JSON.stringify({
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
    }, null, 2),
    language: 'json',
    minimap: { enabled: false }
});

document.getElementById('verify-btn').addEventListener('click', function() {
    let untrusted = JSON.parse(untrustedBlockEditor.getValue());
    let trusted = JSON.parse(trustedBlockEditor.getValue());
    let trustThreshold = document.getElementById('trust-threshold-input').value;
    let trustThresholdParts = trustThreshold.split("/");
    if (trustThresholdParts.length !== 2) {
        window.alert("Expected trust threshold to be of the format \"numerator/denominator\"");
        return;
    }
    let trustThresholdNum = parseInt(trustThresholdParts[0].trim(), 10);
    let trustThresholdDen = parseInt(trustThresholdParts[1].trim(), 10);
    let trustingPeriod = parseInt(document.getElementById('trusting-period-input').value, 10);
    let clockDrift = parseInt(document.getElementById('clock-drift-input').value, 10);
    let now = document.getElementById('now-input').value;

    let options = {
        trust_threshold: [trustThresholdNum, trustThresholdDen],
        trusting_period: trustingPeriod,
        clock_drift: clockDrift
    };

    console.log("Untrusted block:", untrusted);
    console.log("Trusted block:", trusted);
    console.log("Options:", options);
    console.log("Now:", now);

    let verdict = LightClient.verify(untrusted, trusted, options, now);

    document.getElementById('verdict').innerText = JSON.stringify(verdict, null, 2);
    document.getElementById('verdict-section').style.visibility = 'visible';
});
