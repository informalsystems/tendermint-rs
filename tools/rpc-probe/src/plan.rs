//! Coordination of JSON-RPC requests.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::request::Request;
use crate::subscription::Subscription;
use crate::utils::{sanitize, uuid_v4, write_json};
use log::{debug, error, info};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time::Duration;

/// If not specified, the maximum time limit for a subscription interaction
/// (in seconds).
pub const DEFAULT_SUBSCRIPTION_MAX_TIME: u64 = 60;

/// If not specified, the maximum number of events to capture for a
/// subscription prior to terminating successfully.
pub const DEFAULT_SUBSCRIPTION_MAX_EVENTS: usize = 5;

#[derive(Debug, Clone)]
struct PlanConfig {
    // Where to store all the raw JSON outgoing messages.
    out_path: PathBuf,
    // Where to store all the raw JSON incoming messages.
    in_path: PathBuf,
}

/// A structured, sequential execution plan for interactions we would like to
/// execute against a running Tendermint node.
#[derive(Debug, Clone)]
pub struct Plan {
    // Overall configuration for the plan.
    config: PlanConfig,
    // The interactions to execute.
    interactions: Vec<CoordinatedInteractions>,
}

impl Plan {
    /// Create a new plan with the given configuration.
    pub fn new(
        output_path: &Path,
        interactions: impl Into<Vec<CoordinatedInteractions>>,
    ) -> Result<Self> {
        info!(
            "Saving request and response data to: {}",
            output_path.to_str().unwrap()
        );
        let out_path = output_path.join("outgoing");
        let in_path = output_path.join("incoming");
        let paths = vec![&out_path, &in_path];
        for path in paths {
            if !path.exists() {
                fs::create_dir_all(path)?;
                debug!("Created path: {}", path.to_str().unwrap());
            }
            if !path.is_dir() {
                return Err(Error::InvalidParamValue(format!(
                    "path {} should be a directory, but it is not",
                    path.to_str().unwrap()
                )));
            }
        }
        Ok(Self {
            config: PlanConfig { out_path, in_path },
            interactions: interactions.into(),
        })
    }

    /// Executes the plan against a Tendermint node running at the given URL.
    ///
    /// This method assumes that the URL is the full WebSocket URL to the
    /// node's RPC (e.g. `ws://127.0.0.1:26657/websocket`).
    pub async fn execute(&self, url: &str) -> Result<()> {
        info!("Connecting to Tendermint node at {}", url);
        let (mut client, driver) = Client::new(url).await?;
        let driver_handle = tokio::spawn(async move { driver.run().await });

        info!("Executing interactions");
        for interactions in &self.interactions {
            let result =
                execute_interactions(&mut client, &self.config, interactions.clone()).await;
            if let Err(e) = result {
                error!("Failed to execute interaction set: {}", e);
                break;
            }
        }

        info!("Closing connection");
        client.close().await?;
        let _ = driver_handle.await?;
        info!("Connection closed");
        Ok(())
    }
}

/// A set of coordinated planned interactions, which are to be executed either
/// in series or in parallel.
#[derive(Debug, Clone)]
pub enum CoordinatedInteractions {
    Series(Vec<PlannedInteraction>),
    Parallel(Vec<Vec<PlannedInteraction>>),
}

impl From<Vec<PlannedInteraction>> for CoordinatedInteractions {
    fn from(v: Vec<PlannedInteraction>) -> Self {
        Self::Series(v)
    }
}

impl From<Vec<Request>> for CoordinatedInteractions {
    fn from(v: Vec<Request>) -> Self {
        Self::Series(v.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<Vec<PlannedInteraction>>> for CoordinatedInteractions {
    fn from(v: Vec<Vec<PlannedInteraction>>) -> Self {
        Self::Parallel(v)
    }
}

/// A planned interaction is either a simple or complex set of interactions
/// that have some timing/control parameters associated with them.
#[derive(Debug, Clone)]
pub struct PlannedInteraction {
    // The actual interaction itself.
    interaction: Interaction,
    // This name will be used in naming resulting files in which we store
    // request/response data.
    name: String,
    // The maximum time allowable for this planned interaction as a whole
    // (including any waiting to reach a particular height and wait time before
    // interaction execution).
    timeout: Option<Duration>,
    // Wait for this height before executing this interaction.
    min_height: Option<u64>,
    // Wait this much time before executing this interaction.
    pre_wait: Option<Duration>,
    // Whether or not we expect an error from this interaction.
    expect_error: bool,
}

impl PlannedInteraction {
    pub fn new(name: &str, interaction: impl Into<Interaction>) -> Self {
        Self {
            interaction: interaction.into(),
            name: name.to_owned(),
            timeout: None,
            min_height: None,
            pre_wait: None,
            expect_error: false,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_min_height(mut self, h: u64) -> Self {
        self.min_height = Some(h);
        self
    }

    pub fn with_pre_wait(mut self, wait: Duration) -> Self {
        self.pre_wait = Some(wait);
        self
    }

    pub fn expect_error(mut self) -> Self {
        self.expect_error = true;
        self
    }
}

impl From<Request> for PlannedInteraction {
    fn from(r: Request) -> Self {
        let name = r.method.clone();
        Self::new(&name, r)
    }
}

/// We support two types of RPC interaction at present: request/response
/// patterns and subscriptions.
#[derive(Debug, Clone)]
pub enum Interaction {
    Request(Request),
    Subscription(PlannedSubscription),
}

impl From<Request> for Interaction {
    fn from(r: Request) -> Self {
        Interaction::Request(r)
    }
}

impl From<PlannedSubscription> for Interaction {
    fn from(s: PlannedSubscription) -> Self {
        Interaction::Subscription(s)
    }
}

#[derive(Debug, Clone)]
pub struct PlannedSubscription {
    subscription: Subscription,
    max_time: Duration,
    max_events: usize,
}

impl PlannedSubscription {
    pub fn new(subs: impl Into<Subscription>) -> Self {
        Self {
            subscription: subs.into(),
            max_time: Duration::from_secs(DEFAULT_SUBSCRIPTION_MAX_TIME),
            max_events: DEFAULT_SUBSCRIPTION_MAX_EVENTS,
        }
    }

    #[allow(dead_code)]
    pub fn with_max_time(mut self, max_time: Duration) -> Self {
        self.max_time = max_time;
        self
    }

    #[allow(dead_code)]
    pub fn with_max_events(mut self, max_events: usize) -> Self {
        self.max_events = max_events;
        self
    }
}

impl From<PlannedSubscription> for PlannedInteraction {
    fn from(s: PlannedSubscription) -> Self {
        let name = sanitize(&s.subscription.query);
        Self::new(&name, s)
    }
}

pub fn in_series(v: impl Into<Vec<PlannedInteraction>>) -> CoordinatedInteractions {
    CoordinatedInteractions::Series(v.into())
}

pub fn in_parallel(v: impl Into<Vec<Vec<PlannedInteraction>>>) -> CoordinatedInteractions {
    CoordinatedInteractions::Parallel(v.into())
}

async fn execute_interactions(
    client: &mut Client,
    config: &PlanConfig,
    interactions: CoordinatedInteractions,
) -> Result<()> {
    match interactions {
        CoordinatedInteractions::Series(v) => execute_series_interactions(client, config, v).await,
        CoordinatedInteractions::Parallel(v) => {
            execute_parallel_interactions(client, config, v).await
        }
    }
}

async fn execute_series_interactions(
    client: &mut Client,
    config: &PlanConfig,
    interactions: Vec<PlannedInteraction>,
) -> Result<()> {
    for interaction in interactions {
        execute_interaction(client, config, interaction).await?;
    }
    Ok(())
}

async fn execute_parallel_interactions(
    client: &mut Client,
    config: &PlanConfig,
    interaction_sets: Vec<Vec<PlannedInteraction>>,
) -> Result<()> {
    let mut handles = Vec::new();
    for interactions in interaction_sets {
        let mut inner_client = client.clone();
        let inner_config = config.clone();
        let inner_interactions = interactions.clone();
        handles.push(tokio::spawn(async move {
            execute_series_interactions(&mut inner_client, &inner_config, inner_interactions).await
        }));
    }
    // Wait for all tasks to complete, unless one of them fails. If a failure
    // occurs, this will terminate immediately.
    for handle in handles {
        let _ = handle.await?;
    }
    Ok(())
}

async fn execute_interaction(
    client: &mut Client,
    config: &PlanConfig,
    interaction: PlannedInteraction,
) -> Result<()> {
    let inner_interaction = interaction.clone();
    let f = async {
        let _ = &inner_interaction;
        info!("Executing interaction \"{}\"", inner_interaction.name);
        if let Some(wait) = inner_interaction.pre_wait {
            debug!("Sleeping for {} seconds", wait.as_secs_f64());
            tokio::time::sleep(wait).await;
        }
        if let Some(h) = inner_interaction.min_height {
            debug!("Waiting for height {}", h);
            client.wait_for_height(h).await?;
        }
        match inner_interaction.interaction {
            Interaction::Request(request) => {
                execute_request(
                    client,
                    config,
                    &inner_interaction.name,
                    inner_interaction.expect_error,
                    request,
                )
                .await
            }
            Interaction::Subscription(subs) => {
                execute_subscription(
                    client,
                    config,
                    &inner_interaction.name,
                    inner_interaction.expect_error,
                    subs,
                )
                .await
            }
        }
    };
    match interaction.timeout {
        Some(timeout) => tokio::time::timeout(timeout, f).await?,
        None => f.await,
    }
}

async fn execute_request(
    client: &mut Client,
    config: &PlanConfig,
    name: &str,
    expect_error: bool,
    request: Request,
) -> Result<()> {
    let request_json = request.as_json();
    write_json(&config.out_path, name, &request_json).await?;
    let response_json = match client.request(&request_json).await {
        Ok(r) => {
            if expect_error {
                return Err(Error::UnexpectedSuccess);
            }
            r
        }
        Err(e) => match e {
            Error::Failed(_, r) => {
                if !expect_error {
                    return Err(Error::Unexpected(serde_json::to_string_pretty(&r).unwrap()));
                }
                r
            }
            _ => return Err(e),
        },
    };
    write_json(&config.in_path, name, &response_json).await
}

async fn execute_subscription(
    client: &mut Client,
    config: &PlanConfig,
    name: &str,
    expect_error: bool,
    subs: PlannedSubscription,
) -> Result<()> {
    let request_json = subs.subscription.as_json();
    write_json(&config.out_path, name, &request_json).await?;

    let (mut subs_rx, response_json) =
        match client.subscribe(&uuid_v4(), &subs.subscription.query).await {
            Ok(r) => {
                if expect_error {
                    return Err(Error::UnexpectedSuccess);
                }
                r
            }
            Err(e) => match e {
                // We want to capture subscription failures (e.g. malformed
                // queries).
                Error::Failed(_, r) => {
                    if !expect_error {
                        return Err(Error::Unexpected(serde_json::to_string_pretty(&r).unwrap()));
                    }
                    return write_json(&config.in_path, name, &r).await;
                }
                _ => return Err(e),
            },
        };
    write_json(&config.in_path, name, &response_json).await?;

    let timeout = tokio::time::sleep(subs.max_time);
    tokio::pin!(timeout);

    let mut event_count = 0_usize;
    loop {
        tokio::select! {
            _ = &mut timeout => {
                debug!("Subscription reached maximum time limit");
                return Ok(());
            }
            Some(result) = subs_rx.recv() => match result {
                Ok(event_json) => {
                    write_json(
                        &config.in_path,
                        format!("{}_{}", name, event_count).as_str(),
                        &event_json,
                    )
                    .await?;
                    event_count += 1;
                }
                Err(e) => match e {
                    Error::Failed(_, response) => {
                        write_json(
                            &config.in_path,
                            format!("{}_err", name).as_str(),
                            &response,
                        )
                        .await?;
                    },
                    _ => return Err(e),
                }
            }
        }

        if event_count >= subs.max_events {
            debug!(
                "Maximum event limit of {} reached for subscription \"{}\"",
                subs.max_events, name
            );
            return Ok(());
        }
    }
}
