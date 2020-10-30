//! RPC quick probe-related functionality.

use crate::error::Result;
use crate::plan::{Plan, PlanBuilderWithRequests, PlannedRequest};
use crate::request::Request;
use crate::utils::{encode_kvpair, hex_string};
use serde_json::json;
use std::path::Path;
use tokio::time::Duration;

pub fn quick_probe_plan(
    output_path: &Path,
    request_wait: Duration,
) -> Result<PlanBuilderWithRequests> {
    Ok(Plan::new_builder(output_path, request_wait)?
        .then(Request::new("health", json!({})))
        .then(Request::new("abci_info", json!({})))
        .then(PlannedRequest::new(
            "abci_query_non_existent_key",
            Request::new(
                "abci_query",
                json!({
                    "data": hex_string("non-existent-key"),
                }),
            ),
        ))
        .then(Request::new(
            "broadcast_tx_async",
            json!({
                "tx": encode_kvpair("test-async", "async"),
            }),
        ))
        .then(Request::new(
            "broadcast_tx_sync",
            json!({
                "tx": encode_kvpair("test-sync", "sync"),
            }),
        ))
        .then(Request::new(
            "broadcast_tx_commit",
            json!({
                "tx": encode_kvpair("test-commit", "commit"),
            }),
        ))
        .then(PlannedRequest::new(
            "abci_query_existing_key",
            Request::new(
                "abci_query",
                json!({
                    "data": hex_string("test-commit"),
                }),
            ),
        ))
        // Triggers an error from the ABCI app.
        .then(PlannedRequest::new(
            "broadcast_tx_commit_existing_key",
            Request::new(
                "broadcast_tx_commit",
                json!({
                    "tx": encode_kvpair("test-commit", "commit"),
                }),
            ),
        ))
        // Should give us an invalid height error.
        .then(PlannedRequest::new(
            "block_at_height_0",
            Request::new("block", json!({"height": "0"})),
        ))
        .then(PlannedRequest::new(
            "block_at_height_1",
            Request::new("block", json!({"height": "1"})),
        ))
        .then(PlannedRequest::new(
            "block_at_latest_height",
            Request::new("block", json!({})),
        ))
        .then(PlannedRequest::new(
            "block_results_at_height_1",
            Request::new(
                "block_results",
                json!({
                    "height": "1",
                }),
            ),
        ))
        .then(
            PlannedRequest::new(
                "block_results_at_height_10",
                Request::new(
                    "block_results",
                    json!({
                        "height": "10",
                    }),
                ),
            )
            .with_min_height(10),
        ))
}

// Result has the structure: (test name, method, JSON params)
// fn quick_probe_requests<'a>() -> Vec<(&'a str, &'a str, serde_json::Value)> {
//     vec![
//         ("health", "health", json!({})),
//         ("abci_info", "abci_info", json!({})),
//         (
//             "abci_query_non_existent_key",
//             "abci_query",
//             json!({
//                 "data": hex_string("non-existent-key"),
//             }),
//         ),
//         (
//             "broadcast_tx_async",
//             "broadcast_tx_async",
//             json!({
//                 "tx": encode_kvpair("test-async", "async123"),
//             }),
//         ),
//         (
//             "broadcast_tx_sync",
//             "broadcast_tx_sync",
//             json!({
//                 "tx": encode_kvpair("test-sync", "sync123"),
//             }),
//         ),
//         (
//             "broadcast_tx_commit",
//             "broadcast_tx_commit",
//             json!({
//                 "tx": encode_kvpair("test-commit", "commit123"),
//             }),
//         ),
//         (
//             "abci_query_existing_key",
//             "abci_query",
//             json!({
//                 "data": hex_string("test-commit"),
//             }),
//         ),
//         // Should get an error here because we already wrote to this key.
//         (
//             "broadcast_tx_commit_existing_key",
//             "broadcast_tx_commit",
//             json!({
//                 "tx": encode_kvpair("test-commit", "another123"),
//             }),
//         ),
//         (
//             "block_at_height_0",
//             "block",
//             json!({
//                 "height": "0",
//             }),
//         ),
//         (
//             "block_at_height_1",
//             "block",
//             json!({
//                 "height": "1",
//             }),
//         ),
//         ("block_latest", "block", json!({})),
//         (
//             "block_results_height_1",
//             "block_results",
//             json!({
//                 "height": "1",
//             }),
//         ),
//         ("block_results_latest", "block_results", json!({})),
//         // We assume here that at least 5 blocks have been created
//         (
//             "blockchain",
//             "blockchain",
//             json!({
//                 "min_height": "2",
//                 "max_height": "5",
//             }),
//         ),
//     ]
// }
