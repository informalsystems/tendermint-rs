//! ABCI key/value store integration test application.

use futures::StreamExt;
use structopt::StructOpt;
use tendermint_config::net::Address;
use tendermint_rpc::abci::Transaction;
use tendermint_rpc::event::EventData;
use tendermint_rpc::query::EventType;
use tendermint_rpc::{Client, SubscriptionClient, WebSocketClient};
use tokio::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, info};

#[derive(Debug, StructOpt)]
/// A harness for testing tendermint-abci through a full Tendermint node
/// running our in-memory key/value store application (kvstore-rs).
struct Opt {
    /// Tendermint RPC host address.
    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Tendermint RPC port.
    #[structopt(short, long, default_value = "26657")]
    port: u16,

    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: Opt = Opt::from_args();
    tracing_subscriber::fmt()
        .with_max_level(if opt.verbose {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        })
        .init();

    info!("Connecting to Tendermint node at {}:{}", opt.host, opt.port);
    let (mut client, driver) = WebSocketClient::new(Address::Tcp {
        peer_id: None,
        host: opt.host,
        port: opt.port,
    })
    .await?;
    let driver_handle = tokio::spawn(async move { driver.run().await });
    let result = run_tests(&mut client).await;
    client.close()?;
    driver_handle.await??;

    match result {
        Ok(_) => {
            info!("Success!");
            Ok(())
        }
        Err(e) => {
            error!("Test failed: {:?}", e);
            Err(e)
        }
    }
}

async fn run_tests(client: &mut WebSocketClient) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking ABCI application version");
    let abci_info = client.abci_info().await?;
    debug!("Received: {:?}", abci_info);
    if abci_info.data != "kvstore-rs" {
        fail("abci_info", "data", "kvstore-rs", abci_info.data)?;
    }

    info!("Subscribing to transaction events");
    let mut tx_subs = client.subscribe(EventType::Tx.into()).await?;

    info!("Submitting a transaction");
    let raw_tx_key = "test-key".as_bytes().to_vec();
    let raw_tx_value = "test-value".as_bytes().to_vec();
    let mut raw_tx = raw_tx_key.clone();
    raw_tx.push(b'=');
    raw_tx.extend(raw_tx_value.clone());

    let _ = client
        .broadcast_tx_async(Transaction::from(raw_tx.clone()))
        .await?;

    info!("Checking for transaction events");
    let tx = tokio::time::timeout(Duration::from_secs(3), tx_subs.next())
        .await?
        .ok_or_else(|| {
            fail(
                "transaction subscription",
                "transaction",
                "returned",
                "nothing",
            )
            .unwrap_err()
        })??;
    debug!("Got event: {:?}", tx);
    match tx.data {
        EventData::Tx { tx_result } => {
            if tx_result.tx != raw_tx {
                fail("transaction subscription", "tx", raw_tx, tx_result.tx)?;
            }
        }
        _ => fail(
            "transaction subscription",
            "event data",
            "of type Tx",
            tx.data,
        )?,
    }
    // Terminate our transaction subscription
    drop(tx_subs);

    info!("Waiting for at least one block to pass to ensure transaction has been committed");
    let mut new_block_subs = client.subscribe(EventType::NewBlock.into()).await?;
    let _ = new_block_subs.next().await.ok_or_else(|| {
        fail("new block subscription", "event", "returned", "nothing").unwrap_err()
    })??;
    drop(new_block_subs);

    info!(
        "Querying for the value associated with key {}",
        String::from_utf8(raw_tx_key.clone()).unwrap()
    );
    let res = client
        .abci_query(None, raw_tx_key.clone(), None, false)
        .await?;
    if res.key != raw_tx_key {
        fail("abci_query", "key", raw_tx_key, res.key)?;
    }
    if res.value != raw_tx_value {
        fail("abci_query", "value", raw_tx_value, res.value)?;
    }

    Ok(())
}

fn fail<S1, S2, S3, S4>(
    ctx: S1,
    what: S2,
    expected: S3,
    actual: S4,
) -> Result<(), Box<dyn std::error::Error>>
where
    S1: ToString,
    S2: ToString,
    S3: std::fmt::Debug,
    S4: std::fmt::Debug,
{
    Err(Box::new(AssertionError {
        ctx: ctx.to_string(),
        what: what.to_string(),
        expected: format!("{:?}", expected),
        actual: format!("{:?}", actual),
    }))
}

#[derive(Debug)]
struct AssertionError {
    ctx: String,
    what: String,
    expected: String,
    actual: String,
}

impl std::fmt::Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "for {}, expected {} to be {}, but got {}",
            self.ctx, self.what, self.expected, self.actual
        )
    }
}

impl std::error::Error for AssertionError {}
