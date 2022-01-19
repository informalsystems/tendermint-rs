//! Plans for interacting with Tendermint nodes running
//! [Gaia](https://github.com/cosmos/gaia).

use std::path::Path;
use tokio::time::Duration;

use crate::common::*;
use crate::plan::in_series;
use crate::{error::Result, plan::Plan};

/// A simple plan that just queries a Gaia node, without attempting to make any
/// modifications (i.e. without attempting to submit transactions).
pub fn query_plan(output_path: &Path, request_wait: Duration) -> Result<Plan> {
    Plan::new(
        output_path,
        request_wait,
        vec![in_series(vec![
            abci_info(),
            block(0).with_name("block_at_height_0").expect_error(),
            block(1).with_name("block_at_height_1"),
            block(10)
                .with_min_height(10)
                .with_name("block_at_height_10"),
            block_results(10).with_name("block_results_at_height_10"),
            blockchain(1, 10).with_name("blockchain_from_1_to_10"),
            commit(10).with_name("commit_at_height_10"),
            consensus_params(10),
            consensus_state(),
            genesis(),
            net_info(),
            status(),
            subscribe("tm.event = 'NewBlock'").with_name("subscribe_newblock"),
        ])],
    )
}
