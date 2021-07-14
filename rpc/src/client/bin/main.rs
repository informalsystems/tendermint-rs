//! CLI for performing simple interactions against a Tendermint node's RPC.

use std::str::FromStr;
use structopt::StructOpt;
use tendermint::abci::{Path, Transaction};
use tendermint_rpc::{
    error, Client, Error, HttpClient, Paging, Scheme, SubscriptionClient, Url, WebSocketClient,
};
use tracing::level_filters::LevelFilter;
use tracing::{error, info, warn};

/// CLI for performing simple interactions against a Tendermint node's RPC.
///
/// Supports HTTP, HTTPS, WebSocket and secure WebSocket (wss://) URLs.
#[derive(Debug, StructOpt)]
struct Opt {
    /// The URL of the Tendermint node's RPC endpoint.
    #[structopt(
        short,
        long,
        default_value = "http://127.0.0.1:26657",
        env = "TENDERMINT_RPC_URL"
    )]
    url: Url,

    /// An optional HTTP/S proxy through which to submit requests to the
    /// Tendermint node's RPC endpoint. Only available for HTTP/HTTPS endpoints
    /// (i.e. WebSocket proxies are not supported).
    #[structopt(long)]
    proxy_url: Option<Url>,

    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    #[structopt(subcommand)]
    req: Request,
}

#[derive(Debug, StructOpt)]
enum Request {
    #[structopt(flatten)]
    ClientRequest(ClientRequest),
    // TODO(thane): Implement subscription functionality
}

#[derive(Debug, StructOpt)]
enum ClientRequest {
    /// Request information about the ABCI application.
    AbciInfo,
    /// Query the ABCI application.
    AbciQuery {
        /// The path for which you want to query, if any.
        #[structopt(long)]
        path: Option<String>,
        /// The data for which you want to query.
        data: String,
        /// The block height at which to query.
        #[structopt(long)]
        height: Option<u32>,
        #[structopt(long)]
        prove: bool,
    },
    /// Get a block at a given height.
    Block { height: u32 },
    /// Get block headers between two heights (min <= height <= max).
    Blockchain {
        /// The minimum height
        min: u32,
        /// The maximum height.
        max: u32,
    },
    /// Request the block results at a given height.
    BlockResults {
        /// The height of the block you want.
        height: u32,
    },
    // TODO(thane): Implement evidence broadcast
    /// Broadcast a transaction asynchronously (without waiting for the ABCI
    /// app to check it or for it to be committed).
    BroadcastTxAsync {
        /// The transaction to broadcast.
        tx: String,
    },
    /// Broadcast a transaction, waiting for it to be fully committed before
    /// returning.
    BroadcastTxCommit {
        /// The transaction to broadcast.
        tx: String,
    },
    /// Broadcast a transaction synchronously (waiting for the ABCI app to
    /// check it, but not for it to be committed).
    BroadcastTxSync {
        /// The transaction to broadcast.
        tx: String,
    },
    /// Get the commit for the given height.
    Commit { height: u32 },
    /// Get the current consensus state.
    ConsensusState,
    /// Get the node's genesis data.
    Genesis,
    /// Get the node's health.
    Health,
    /// Request the latest block.
    LatestBlock,
    /// Request the results for the latest block.
    LatestBlockResults,
    /// Request the latest commit.
    LatestCommit,
    /// Obtain information about the P2P stack and other network connections.
    NetInfo,
    /// Get Tendermint status (node info, public key, latest block hash, etc.).
    Status,
    // TODO(thane): Implement txsearch endpoint.
    /// Get the validators at the given height.
    Validators {
        /// The height at which to query the validators.
        height: u32,
        /// Fetch all validators.
        #[structopt(long)]
        all: bool,
        /// The page of validators to retrieve.
        #[structopt(long)]
        page: Option<usize>,
        /// The number of validators to retrieve per page.
        #[structopt(long)]
        per_page: Option<u8>,
    },
}

#[tokio::main]
async fn main() {
    let opt: Opt = Opt::from_args();
    let log_level = if opt.verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    // All our logging goes to stderr, so our output can go to stdout
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .init();

    let proxy_url = match get_http_proxy_url(opt.url.scheme(), opt.proxy_url.clone()) {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to obtain proxy URL: {}", e);
            std::process::exit(-1);
        }
    };
    let result = match opt.url.scheme() {
        Scheme::Http | Scheme::Https => http_request(opt.url, proxy_url, opt.req).await,
        Scheme::WebSocket | Scheme::SecureWebSocket => match opt.proxy_url {
            Some(_) => Err(error::invalid_params_error(
                "proxies are only supported for use with HTTP clients at present".to_string(),
            )),
            None => websocket_request(opt.url, opt.req).await,
        },
    };
    if let Err(e) = result {
        error!("Failed: {}", e);
        std::process::exit(-1);
    }
}

// Retrieve the proxy URL with precedence:
// 1. If supplied, that's the proxy URL used.
// 2. If not supplied, but environment variable HTTP_PROXY or HTTPS_PROXY are
//    supplied, then use the appropriate variable for the URL in question.
fn get_http_proxy_url(url_scheme: Scheme, proxy_url: Option<Url>) -> Result<Option<Url>, Error> {
    match proxy_url {
        Some(u) => Ok(Some(u)),
        None => match url_scheme {
            Scheme::Http => std::env::var("HTTP_PROXY").ok(),
            Scheme::Https => std::env::var("HTTPS_PROXY")
                .ok()
                .or_else(|| std::env::var("HTTP_PROXY").ok()),
            _ => {
                if std::env::var("HTTP_PROXY").is_ok() || std::env::var("HTTPS_PROXY").is_ok() {
                    warn!(
                        "Ignoring HTTP proxy environment variables for non-HTTP client connection"
                    );
                }
                None
            }
        }
        .map(|u| u.parse())
        .transpose(),
    }
}

async fn http_request(url: Url, proxy_url: Option<Url>, req: Request) -> Result<(), Error> {
    let client = match proxy_url {
        Some(proxy_url) => {
            info!(
                "Using HTTP client with proxy {} to submit request to {}",
                proxy_url, url
            );
            HttpClient::new_with_proxy(url, proxy_url)
        }
        None => {
            info!("Using HTTP client to submit request to: {}", url);
            HttpClient::new(url)
        }
    }?;

    match req {
        Request::ClientRequest(r) => client_request(&client, r).await,
    }
}

async fn websocket_request(url: Url, req: Request) -> Result<(), Error> {
    info!("Using WebSocket client to submit request to: {}", url);
    let (client, driver) = WebSocketClient::new(url).await?;
    let driver_hdl = tokio::spawn(async move { driver.run().await });

    let result = match req {
        Request::ClientRequest(r) => client_request(&client, r).await,
    };

    client.close()?;
    driver_hdl.await.map_err(error::join_error)??;
    result
}

async fn client_request<C>(client: &C, req: ClientRequest) -> Result<(), Error>
where
    C: Client + Sync,
{
    let result = match req {
        ClientRequest::AbciInfo => {
            serde_json::to_string_pretty(&client.abci_info().await?).map_err(error::serde_error)?
        }
        ClientRequest::AbciQuery {
            path,
            data,
            height,
            prove,
        } => serde_json::to_string_pretty(
            &client
                .abci_query(
                    path.map(|s| Path::from_str(&s))
                        .transpose()
                        .map_err(error::tendermint_error)?,
                    data,
                    height.map(Into::into),
                    prove,
                )
                .await?,
        )
        .map_err(error::serde_error)?,
        ClientRequest::Block { height } => {
            serde_json::to_string_pretty(&client.block(height).await?)
                .map_err(error::serde_error)?
        }
        ClientRequest::Blockchain { min, max } => {
            serde_json::to_string_pretty(&client.blockchain(min, max).await?)
                .map_err(error::serde_error)?
        }
        ClientRequest::BlockResults { height } => {
            serde_json::to_string_pretty(&client.block_results(height).await?)
                .map_err(error::serde_error)?
        }
        ClientRequest::BroadcastTxAsync { tx } => serde_json::to_string_pretty(
            &client
                .broadcast_tx_async(Transaction::from(tx.into_bytes()))
                .await?,
        )
        .map_err(error::serde_error)?,
        ClientRequest::BroadcastTxCommit { tx } => serde_json::to_string_pretty(
            &client
                .broadcast_tx_commit(Transaction::from(tx.into_bytes()))
                .await?,
        )
        .map_err(error::serde_error)?,
        ClientRequest::BroadcastTxSync { tx } => serde_json::to_string_pretty(
            &client
                .broadcast_tx_sync(Transaction::from(tx.into_bytes()))
                .await?,
        )
        .map_err(error::serde_error)?,
        ClientRequest::Commit { height } => {
            serde_json::to_string_pretty(&client.commit(height).await?)
                .map_err(error::serde_error)?
        }
        ClientRequest::LatestBlock => serde_json::to_string_pretty(&client.latest_block().await?)
            .map_err(error::serde_error)?,
        ClientRequest::LatestBlockResults => {
            serde_json::to_string_pretty(&client.latest_block_results().await?)
                .map_err(error::serde_error)?
        }
        ClientRequest::LatestCommit => serde_json::to_string_pretty(&client.latest_commit().await?)
            .map_err(error::serde_error)?,
        ClientRequest::ConsensusState => {
            serde_json::to_string_pretty(&client.consensus_state().await?)
                .map_err(error::serde_error)?
        }
        ClientRequest::Genesis => {
            serde_json::to_string_pretty(&client.genesis().await?).map_err(error::serde_error)?
        }
        ClientRequest::Health => {
            serde_json::to_string_pretty(&client.health().await?).map_err(error::serde_error)?
        }
        ClientRequest::NetInfo => {
            serde_json::to_string_pretty(&client.net_info().await?).map_err(error::serde_error)?
        }
        ClientRequest::Status => {
            serde_json::to_string_pretty(&client.status().await?).map_err(error::serde_error)?
        }
        ClientRequest::Validators {
            height,
            all,
            page,
            per_page,
        } => {
            let paging = if all {
                Paging::All
            } else {
                match page.zip(per_page) {
                    Some((page, per_page)) => Paging::Specific {
                        page_number: page.into(),
                        per_page: per_page.into(),
                    },
                    None => Paging::Default,
                }
            };
            serde_json::to_string_pretty(&client.validators(height, paging).await?)
                .map_err(error::serde_error)?
        }
    };

    println!("{}", result);
    Ok(())
}
