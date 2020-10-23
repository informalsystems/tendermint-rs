//! RPC quick probe-related functionality.

use crate::error::Result;
use crate::messages::request_wrapper;
use crate::opts::GeneralOptions;
use crate::websocket::WebSocketClient;
use serde_json::json;
use tracing::info;

/// Execute a quick probe against a live Tendermint node using the given
/// options.
///
/// This assumes that the given Tendermint node is running the `kvstore`
/// application.
pub async fn quick_probe(opts: GeneralOptions) -> Result<()> {
    info!(
        "Connecting to Tendermint node WebSocket RPC endpoint: {}",
        opts.addr
    );
    let (mut client, driver) = WebSocketClient::new(&opts.addr).await?;
    let driver_handle = tokio::spawn(async move { driver.run().await });

    let abci_info = client
        .request(request_wrapper("abci_info", json!({})))
        .await?;
    info!(
        "Received ABCI info: {}",
        serde_json::to_string_pretty(&abci_info).unwrap()
    );

    info!("Closing RPC connection");
    client.close().await?;
    driver_handle.await??;
    info!("Connection closed");
    Ok(())
}
