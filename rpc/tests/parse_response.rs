//! Tendermint RPC endpoint testing.

use std::{fs, path::PathBuf};
use tendermint_rpc::abci::Code;

use core::str::FromStr;
use tendermint::vote;
use tendermint_rpc::endpoint::consensus_state::RoundVote;
use tendermint_rpc::{
    endpoint,
    error::{Error, ErrorDetail},
    Code as RpcCode, Response,
};

const EXAMPLE_APP: &str = "GaiaApp";
const EXAMPLE_CHAIN: &str = "cosmoshub-2";

fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json")).unwrap()
}

#[test]
fn abci_info() {
    let response = endpoint::abci_info::Response::from_string(&read_json_fixture("abci_info"))
        .unwrap()
        .response;

    assert_eq!(response.data.as_str(), EXAMPLE_APP);
    assert_eq!(response.last_block_height.value(), 488_120);
}

#[test]
fn abci_query() {
    let response = endpoint::abci_query::Response::from_string(&read_json_fixture("abci_query"))
        .unwrap()
        .response;

    assert_eq!(response.height.value(), 1);
    assert!(response.proof.is_some());
    let proof = response.proof.unwrap();
    assert_eq!(proof.ops.len(), 2);
    assert_eq!(proof.ops[0].field_type, "iavl:v");
    assert_eq!(proof.ops[1].field_type, "multistore");
}

#[test]
fn block() {
    let response = endpoint::block::Response::from_string(&read_json_fixture("block")).unwrap();

    assert_eq!(response.block.header.version.block, 10);
    assert_eq!(response.block.header.chain_id.as_str(), EXAMPLE_CHAIN);
    assert_eq!(response.block.header.height.value(), 10);
    assert_eq!(response.block.data.iter().len(), 0);
    assert_eq!(response.block.evidence.iter().len(), 0);
    assert_eq!(
        response
            .block
            .last_commit
            .as_ref()
            .unwrap()
            .signatures
            .len(),
        1
    );
}

#[test]
fn block_with_evidences() {
    let response =
        endpoint::block::Response::from_string(&read_json_fixture("block_with_evidences")).unwrap();

    let evidence = response.block.evidence.iter().next().unwrap();

    match evidence {
        tendermint::evidence::Evidence::DuplicateVote(_) => (),
        _ => unreachable!(),
    }
}

// TODO: Update this test and its json file
// #[test]
// fn block_empty_block_id() {
//     let response =
//         endpoint::block::Response::from_string(&read_json_fixture("block_empty_block_id"))
//             .unwrap();
//
//     let tendermint::Block { last_commit, .. } = response.block;
//
//     assert_eq!(last_commit.as_ref().unwrap().precommits.len(), 2);
//     assert!(last_commit.unwrap().precommits[0]
//         .as_ref()
//         .unwrap()
//         .block_id
//         .is_none());
// }

#[test]
fn first_block() {
    let response =
        endpoint::block::Response::from_string(&read_json_fixture("first_block")).unwrap();

    assert_eq!(response.block.header.version.block, 10);
    assert_eq!(response.block.header.chain_id.as_str(), EXAMPLE_CHAIN);
    assert_eq!(response.block.header.height.value(), 1);
    assert!(response.block.header.last_block_id.is_none());

    assert_eq!(response.block.data.iter().len(), 0);
    assert_eq!(response.block.evidence.iter().len(), 0);
    assert!(response.block.last_commit.is_none());
}
#[test]
fn block_results() {
    let response =
        endpoint::block_results::Response::from_string(&read_json_fixture("block_results"))
            .unwrap();
    assert_eq!(response.height.value(), 1814);

    let validator_updates = response.validator_updates;
    let deliver_tx = response.txs_results.unwrap();
    let log_json = deliver_tx[0].log.value();
    let log_json_value = serde_json::Value::from_str(log_json.as_str()).unwrap();

    assert_eq!(log_json_value[0]["msg_index"].as_str().unwrap(), "0");
    assert!(log_json_value[0]["success"].as_bool().unwrap());

    assert_eq!(deliver_tx[0].gas_wanted.value(), 200_000);
    assert_eq!(deliver_tx[0].gas_used.value(), 105_662);
    assert_eq!(deliver_tx[0].events.len(), 1);
    assert_eq!(deliver_tx[0].events[0].attributes.len(), 3);
    assert_eq!(deliver_tx[0].events[0].attributes[0].key.as_ref(), "action");
    assert_eq!(
        deliver_tx[0].events[0].attributes[0].value.as_ref(),
        "delegate"
    );

    assert_eq!(validator_updates[0].power.value(), 1_233_243);
}

#[test]
fn blockchain() {
    let response =
        endpoint::blockchain::Response::from_string(&read_json_fixture("blockchain")).unwrap();

    assert_eq!(response.last_height.value(), 488_556);
    assert_eq!(response.block_metas.len(), 10);

    let block_meta = &response.block_metas[0];
    assert_eq!(block_meta.header.chain_id.as_str(), EXAMPLE_CHAIN)
}

#[test]
fn broadcast_tx_async() {
    let response = endpoint::broadcast::tx_async::Response::from_string(&read_json_fixture(
        "broadcast_tx_async",
    ))
    .unwrap();

    assert_eq!(
        &response.hash.to_string(),
        "88D4266FD4E6338D13B845FCF289579D209C897823B9217DA3E161936F031589"
    );
}

#[test]
fn broadcast_tx_sync() {
    let response = endpoint::broadcast::tx_sync::Response::from_string(&read_json_fixture(
        "broadcast_tx_sync",
    ))
    .unwrap();

    assert_eq!(response.code, Code::Ok);
    assert_eq!(
        &response.hash.to_string(),
        "88D4266FD4E6338D13B845FCF289579D209C897823B9217DA3E161936F031589"
    );
}

#[test]
fn broadcast_tx_sync_int() {
    let response = endpoint::broadcast::tx_sync::Response::from_string(&read_json_fixture(
        "broadcast_tx_sync_int",
    ))
    .unwrap();

    assert_eq!(response.code, Code::Ok);
    assert_eq!(
        &response.hash.to_string(),
        "88D4266FD4E6338D13B845FCF289579D209C897823B9217DA3E161936F031589"
    );
}

#[test]
fn broadcast_tx_commit() {
    let response = endpoint::broadcast::tx_commit::Response::from_string(&read_json_fixture(
        "broadcast_tx_commit",
    ))
    .unwrap();

    assert_eq!(
        response.deliver_tx.data.unwrap().value(),
        &vec![
            10, 22, 10, 20, 99, 111, 110, 110, 101, 99, 116, 105, 111, 110, 95, 111, 112, 101, 110,
            95, 105, 110, 105, 116
        ]
    );
    assert_eq!(
        &response.hash.to_string(),
        "EFA00D85332A8197CF290E4724BAC877EA93DDFE547A561828BAE45A29BF1DAD"
    );
    assert_eq!(5, response.deliver_tx.events.len());
}

#[test]
fn broadcast_tx_commit_null_data() {
    let response = endpoint::broadcast::tx_commit::Response::from_string(&read_json_fixture(
        "broadcast_tx_commit_null_data",
    ))
    .unwrap();

    assert_eq!(
        &response.hash.to_string(),
        "88D4266FD4E6338D13B845FCF289579D209C897823B9217DA3E161936F031589"
    );
}

#[test]
fn commit() {
    let response = endpoint::commit::Response::from_string(&read_json_fixture("commit")).unwrap();
    let header = response.signed_header.header;
    assert_eq!(header.chain_id.as_ref(), "dockerchain");
    // For now we just want to make sure the commit including precommits and a block_id exist
    // in SignedHeader; later we should verify some properties: e.g. block_id.hash matches the
    // header etc:
    let commit = response.signed_header.commit;
    let block_id = commit.block_id;
    let _signatures = &commit.signatures;
    assert_eq!(header.hash(), block_id.hash);
}

#[test]
fn commit_height_1() {
    let response = endpoint::commit::Response::from_string(&read_json_fixture("commit_1")).unwrap();
    let header = response.signed_header.header;
    let commit = response.signed_header.commit;
    let block_id = commit.block_id;
    assert_eq!(header.hash(), block_id.hash);
}

#[test]
fn genesis() {
    let response = endpoint::genesis::Response::from_string(&read_json_fixture("genesis")).unwrap();

    let tendermint::Genesis {
        chain_id,
        consensus_params,
        ..
    } = response.genesis;

    assert_eq!(chain_id.as_str(), EXAMPLE_CHAIN);
    assert_eq!(consensus_params.block.max_bytes, 200_000);
}

#[test]
fn health() {
    endpoint::health::Response::from_string(&read_json_fixture("health")).unwrap();
}

#[test]
fn net_info() {
    let response =
        endpoint::net_info::Response::from_string(&read_json_fixture("net_info")).unwrap();

    assert_eq!(response.n_peers, 2);
    assert_eq!(response.peers[0].node_info.network.as_str(), EXAMPLE_CHAIN);
}

#[test]
fn status() {
    let response = endpoint::status::Response::from_string(&read_json_fixture("status")).unwrap();

    assert_eq!(response.node_info.network.as_str(), EXAMPLE_CHAIN);
    assert_eq!(response.sync_info.latest_block_height.value(), 410_744);
    assert_eq!(response.validator_info.power.value(), 0);
}

#[test]
fn validators() {
    let response =
        endpoint::validators::Response::from_string(&read_json_fixture("validators")).unwrap();

    assert_eq!(response.block_height.value(), 42);

    let validators = response.validators;
    assert_eq!(validators.len(), 65);
}

#[test]
fn jsonrpc_error() {
    let result = endpoint::blockchain::Response::from_string(&read_json_fixture("error"));

    match result {
        Err(Error(ErrorDetail::Response(e), _)) => {
            let response = e.source;
            assert_eq!(response.code(), RpcCode::InternalError);
            assert_eq!(response.message(), "Internal error");
            assert_eq!(
                response.data().unwrap(),
                "min height 321 can't be greater than max height 123"
            );
        }
        _ => panic!("expected Response error"),
    }
}

#[test]
fn tx_no_prove() {
    let tx = endpoint::tx::Response::from_string(&read_json_fixture("tx_no_prove")).unwrap();

    assert_eq!(
        "291B44C883803751917D547238EAC419E968C0171A3154D777B2EA8EA5039C57",
        tx.hash.to_string()
    );
    assert_eq!(2, tx.height.value());

    let events = &tx.tx_result.events;
    assert_eq!(events.len(), 6);
    assert_eq!(events[0].attributes.len(), 3);
    assert_eq!(events[0].attributes[0].key.as_ref(), "recipient");
    assert_eq!(
        events[0].attributes[0].value.as_ref(),
        "cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta"
    );

    assert!(tx.proof.is_none());
}

#[test]
fn tx_with_prove() {
    let tx = endpoint::tx::Response::from_string(&read_json_fixture("tx_with_prove")).unwrap();

    assert_eq!(
        "291B44C883803751917D547238EAC419E968C0171A3154D777B2EA8EA5039C57",
        tx.hash.to_string()
    );
    assert_eq!(2, tx.height.value());

    let events = &tx.tx_result.events;
    assert_eq!(events.len(), 6);
    assert_eq!(events[0].attributes.len(), 3);
    assert_eq!(events[0].attributes[0].key.as_ref(), "recipient");
    assert_eq!(
        events[0].attributes[0].value.as_ref(),
        "cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta"
    );

    let proof = tx.proof.as_ref().unwrap();
    assert_eq!(
        vec![
            10, 159, 1, 10, 142, 1, 10, 28, 47, 99, 111, 115, 109, 111, 115, 46, 98, 97, 110, 107,
            46, 118, 49, 98, 101, 116, 97, 49, 46, 77, 115, 103, 83, 101, 110, 100, 18, 110, 10,
            45, 99, 111, 115, 109, 111, 115, 49, 115, 50, 116, 119, 52, 53, 99, 55, 115, 116, 115,
            97, 102, 107, 52, 50, 118, 115, 122, 57, 115, 106, 48, 57, 106, 109, 48, 57, 121, 54,
            116, 107, 52, 113, 101, 101, 114, 104, 18, 45, 99, 111, 115, 109, 111, 115, 49, 110,
            118, 51, 117, 102, 55, 104, 112, 117, 118, 107, 52, 101, 109, 51, 57, 118, 120, 114,
            57, 52, 52, 104, 112, 104, 117, 106, 116, 117, 113, 97, 50, 120, 108, 55, 54, 56, 56,
            26, 14, 10, 9, 115, 97, 109, 111, 108, 101, 97, 110, 115, 18, 1, 49, 18, 9, 116, 101,
            115, 116, 32, 109, 101, 109, 111, 24, 169, 70, 18, 102, 10, 78, 10, 70, 10, 31, 47, 99,
            111, 115, 109, 111, 115, 46, 99, 114, 121, 112, 116, 111, 46, 115, 101, 99, 112, 50,
            53, 54, 107, 49, 46, 80, 117, 98, 75, 101, 121, 18, 35, 10, 33, 3, 98, 211, 158, 175,
            190, 7, 170, 66, 0, 20, 131, 204, 81, 56, 214, 191, 143, 101, 195, 149, 126, 234, 114,
            55, 58, 237, 26, 39, 95, 114, 111, 164, 18, 4, 10, 2, 8, 1, 18, 20, 10, 14, 10, 9, 115,
            97, 109, 111, 108, 101, 97, 110, 115, 18, 1, 49, 16, 160, 141, 6, 26, 64, 185, 213,
            205, 42, 231, 20, 240, 14, 103, 188, 41, 94, 116, 55, 181, 30, 185, 212, 221, 131, 145,
            132, 32, 83, 223, 255, 85, 10, 220, 211, 124, 172, 29, 152, 55, 91, 199, 85, 165, 186,
            68, 87, 22, 14, 235, 208, 43, 62, 93, 129, 228, 237, 222, 77, 146, 245, 107, 123, 173,
            19, 73, 154, 174, 249
        ],
        proof.data
    );
    assert_eq!(
        vec![
            105, 196, 2, 216, 75, 198, 114, 80, 111, 27, 54, 17, 4, 107, 139, 37, 40, 156, 38, 0,
            253, 122, 0, 118, 137, 197, 148, 154, 51, 32, 101, 87
        ],
        proof.root_hash
    );
}

#[test]
fn tx_search_no_prove() {
    let response =
        endpoint::tx_search::Response::from_string(&read_json_fixture("tx_search_no_prove"))
            .unwrap();

    assert_eq!(8, response.total_count);
    assert_eq!(8, response.txs.len());
    assert_eq!(
        "9F28904F9C0F3AB74A81CBA48E39124DA1C680B47FBFCBA0126870DB722BCC30",
        response.txs[0].hash.to_string()
    );
    assert_eq!(11, response.txs[0].height.value());
    assert!(response.txs[0].proof.is_none());

    let events = &response.txs[0].tx_result.events;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].attributes.len(), 4);
    assert_eq!(events[0].attributes[0].key.as_ref(), "creator");
    assert_eq!(events[0].attributes[0].value.as_ref(), "Cosmoshi Netowoko");
}

#[test]
fn tx_search_with_prove() {
    let response =
        endpoint::tx_search::Response::from_string(&read_json_fixture("tx_search_with_prove"))
            .unwrap();

    assert_eq!(8, response.total_count);
    assert_eq!(8, response.txs.len());
    assert_eq!(
        "9F28904F9C0F3AB74A81CBA48E39124DA1C680B47FBFCBA0126870DB722BCC30",
        response.txs[0].hash.to_string()
    );
    assert_eq!(11, response.txs[0].height.value());
    let proof = response.txs[0].proof.as_ref().unwrap();
    assert_eq!(
        vec![97, 115, 121, 110, 99, 45, 107, 101, 121, 61, 118, 97, 108, 117, 101],
        proof.data
    );
    assert_eq!(
        vec![
            245, 70, 67, 176, 5, 16, 101, 200, 125, 163, 26, 101, 69, 49, 182, 95, 155, 87, 56, 15,
            155, 243, 51, 47, 245, 188, 167, 88, 69, 103, 38, 140
        ],
        proof.root_hash
    );

    let events = &response.txs[0].tx_result.events;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].attributes.len(), 4);
    assert_eq!(events[0].attributes[0].key.as_ref(), "creator");
    assert_eq!(events[0].attributes[0].value.as_ref(), "Cosmoshi Netowoko");
}

#[test]
fn consensus_state() {
    let response =
        endpoint::consensus_state::Response::from_string(&read_json_fixture("consensus_state"))
            .unwrap();

    let hrs = &response.round_state.height_round_step;
    assert_eq!(hrs.height.value(), 1262197);
    assert_eq!(hrs.round.value(), 0);
    assert_eq!(hrs.step, 8);

    let hvs = &response.round_state.height_vote_set;
    assert_eq!(hvs.len(), 1);
    assert_eq!(hvs[0].round, 0);
    assert_eq!(hvs[0].prevotes.len(), 2);
    match &hvs[0].prevotes[0] {
        RoundVote::Vote(summary) => {
            assert_eq!(summary.validator_index, 0);
            assert_eq!(
                summary.validator_address_fingerprint.as_ref(),
                vec![0, 0, 1, 228, 67, 253]
            );
            assert_eq!(summary.height.value(), 1262197);
            assert_eq!(summary.round.value(), 0);
            assert_eq!(summary.vote_type, vote::Type::Prevote);
            assert_eq!(
                summary.block_id_hash_fingerprint.as_ref(),
                vec![99, 74, 218, 241, 244, 2]
            );
            assert_eq!(
                summary.signature_fingerprint.as_ref(),
                vec![123, 185, 116, 225, 186, 64]
            );
            assert_eq!(
                summary.timestamp.as_rfc3339(),
                "2019-08-01T11:52:35.513572509Z"
            );
        }
        _ => panic!("unexpected round vote type: {:?}", hvs[0].prevotes[0]),
    }
    assert_eq!(hvs[0].prevotes[1], RoundVote::Nil);
    assert_eq!(hvs[0].precommits.len(), 2);
    match &hvs[0].precommits[0] {
        RoundVote::Vote(summary) => {
            assert_eq!(summary.validator_index, 5);
            assert_eq!(
                summary.validator_address_fingerprint.as_ref(),
                vec![24, 199, 141, 19, 92, 157]
            );
            assert_eq!(summary.height.value(), 1262197);
            assert_eq!(summary.round.value(), 0);
            assert_eq!(summary.vote_type, vote::Type::Precommit);
            assert_eq!(
                summary.block_id_hash_fingerprint.as_ref(),
                vec![99, 74, 218, 241, 244, 2]
            );
            assert_eq!(
                summary.signature_fingerprint.as_ref(),
                vec![139, 94, 255, 254, 171, 205]
            );
            assert_eq!(
                summary.timestamp.as_rfc3339(),
                "2019-08-01T11:52:36.25600005Z"
            );
        }
        _ => panic!("unexpected round vote type: {:?}", hvs[0].precommits[0]),
    }
    assert_eq!(hvs[0].precommits[1], RoundVote::Nil);
}
