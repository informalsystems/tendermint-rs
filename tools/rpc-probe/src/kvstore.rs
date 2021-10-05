//! RPC quick probe-related functionality specifically targeting a Tendermint
//! node running the `kvstore` ABCI application.

use crate::common::*;
use crate::error::Result;
use crate::plan::{in_parallel, in_series, Plan};
use std::path::Path;
use tokio::time::Duration;

pub fn quick_probe_plan(output_path: &Path, request_wait: Duration) -> Result<Plan> {
    Plan::new(
        output_path,
        request_wait,
        vec![
            in_series(vec![
                abci_info(),
                abci_query("non_existent_key").with_name("abci_query_with_non_existent_key"),
                block(0).with_name("block_at_height_0").expect_error(),
                block(1).with_name("block_at_height_1"),
                block(10)
                    .with_min_height(10)
                    .with_name("block_at_height_10"),
                block_results(10).with_name("block_results_at_height_10"),
                block_by_hash("0x00112233445566778899AABBCCDDEEFF00112233445566778899AABBCCDDEEFF").with_name("block_by_hash"),
                block_search("block.height > 1", 1, 10, "asc").with_name("block_search"),
                blockchain(1, 10).with_name("blockchain_from_1_to_10"),
                commit(10).with_name("commit_at_height_10"),
                consensus_params(10),
                consensus_state(),
                broadcast_tx("async", "async-key", "value"),
                broadcast_tx("sync", "sync-key", "value"),
                broadcast_tx("commit", "commit-key", "value"),
                genesis(),
                net_info(),
                status(),
                subscribe("tm.event = 'NewBlock'").with_name("subscribe_newblock"),
                subscribe("malformed query")
                    .with_name("subscribe_malformed")
                    .expect_error(),
            ]),
            // Here we subscribe to receive incoming transaction events, and we
            // simultaneously send a sequential bunch of transactions.
            in_parallel(vec![
                vec![subscribe("tm.event = 'Tx'").with_name("subscribe_txs")],
                (0..=5)
                    .into_iter()
                    .map(|i| {
                        broadcast_tx("async", format!("tx{}", i).as_str(), "value")
                            .with_name(format!("subscribe_txs_broadcast_tx_{}", i).as_str())
                            .with_pre_wait(Duration::from_millis(500))
                    })
                    .collect(),
            ]),
            in_series(vec![
                // This should have been created in the previous set of
                // interactions.
                abci_query("tx0").with_name("abci_query_with_existing_key"),
                tx(
                    "FCB86F71C4EFF43E13C51FA12791F6DD1DDB8600A51131BE2289614D6882F6BE",
                    false,
                )
                .with_name("tx_no_prove"),
                tx(
                    "FCB86F71C4EFF43E13C51FA12791F6DD1DDB8600A51131BE2289614D6882F6BE",
                    true,
                )
                .with_name("tx_prove"),
                tx_search("tx.height > 1", false, 1, 10, "asc").with_name("tx_search_no_prove"),
                tx_search("tx.height > 1", true, 1, 10, "asc").with_name("tx_search_with_prove"),
            ]),
        ],
    )
<<<<<<< HEAD
=======
    .into()
}

pub fn block(height: u64) -> PlannedInteraction {
    Request::new(
        "block",
        json!({
            "height": format!("{}", height),
        }),
    )
    .into()
}

pub fn block_search(query: &str, page: u32, per_page: u32, order_by: &str) -> PlannedInteraction {
    Request::new(
        "block_search",
        json!({
            "query": query,
            "page": format!("{}", page),
            "per_page": format!("{}", per_page),
            "order_by": order_by,
        }),
    )
    .into()
}

pub fn block_results(height: u64) -> PlannedInteraction {
    Request::new(
        "block_results",
        json!({
            "height": format!("{}", height),
        }),
    )
    .into()
}

pub fn blockchain(min_height: u64, max_height: u64) -> PlannedInteraction {
    Request::new(
        "blockchain",
        json!({
            "minHeight": format!("{}", min_height),
            "maxHeight": format!("{}", max_height),
        }),
    )
    .into()
}

pub fn broadcast_tx(method: &str, key: &str, value: &str) -> PlannedInteraction {
    Request::new(
        format!("broadcast_tx_{}", method).as_str(),
        json!({
            "tx": encode_kvpair(key, value),
        }),
    )
    .into()
}

pub fn commit(height: u64) -> PlannedInteraction {
    Request::new(
        "commit",
        json!({
            "height": format!("{}", height),
        }),
    )
    .into()
}

pub fn consensus_params(height: u64) -> PlannedInteraction {
    Request::new(
        "consensus_params",
        json!({
            "height": format!("{}", height),
        }),
    )
    .into()
}

pub fn consensus_state() -> PlannedInteraction {
    Request::new("consensus_state", json!(null)).into()
}

pub fn genesis() -> PlannedInteraction {
    Request::new("genesis", json!(null)).into()
}

pub fn net_info() -> PlannedInteraction {
    Request::new("net_info", json!(null)).into()
}

pub fn status() -> PlannedInteraction {
    Request::new("status", json!(null)).into()
}

pub fn subscribe(query: &str) -> PlannedInteraction {
    PlannedSubscription::new(query).into()
}

pub fn tx_search(
    query: &str,
    prove: bool,
    page: u32,
    per_page: u8,
    order_by: &str,
) -> PlannedInteraction {
    Request::new(
        "tx_search",
        json!({
            "query": query,
            "prove": prove,
            "page": format!("{}", page),
            "per_page": format!("{}", per_page),
            "order_by": order_by,
        }),
    )
    .into()
>>>>>>> 38d91a59 (light-client: Replace Io trait with async variant)
}
