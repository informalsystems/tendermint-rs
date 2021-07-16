use futures_util::StreamExt;

use tendermint_rpc::{
    query::{EventType, Query},
    Result, SubscriptionClient, Url, WebSocketClient,
};

fn register_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    register_tracing();

    let addr: Url = "ws://127.0.0.1:26657/websocket".parse().unwrap();
    let (client, driver) = WebSocketClient::new(addr).await?;

    tokio::spawn(async move { driver.run().await });

    let mut new_blocks = client
        .subscribe(Query::from(EventType::NewBlock))
        .await?
        .boxed();

    let mut txs = client.subscribe(Query::from(EventType::Tx)).await?.boxed();

    loop {
        tokio::select! {
            Some(new_block) = new_blocks.next() => {
                dbg!(new_block);
            }
            Some(tx) = txs.next() => {
                dbg!(tx);
            }
        }
    }

    Ok(())
}
