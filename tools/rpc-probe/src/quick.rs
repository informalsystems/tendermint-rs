//! RPC quick probe-related functionality.

use crate::error::Result;
use crate::kvstore::*;
use crate::plan::{in_parallel, in_series, Plan};
use std::path::Path;
use tokio::time::Duration;

pub fn quick_probe_plan(output_path: &Path, request_wait: Duration) -> Result<Plan> {
    Plan::new(
        "quick-probe",
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
}
