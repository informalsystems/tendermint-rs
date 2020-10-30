//! Coordination of JSON-RPC requests.

use crate::error::{Error, Result};
use crate::request::Request;
use crate::utils::uuid_v4;
use crate::websocket::{WebSocketClient, WebSocketClientDriver};
use futures::StreamExt;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time::Duration;

/// A structured, sequential execution plan for requests which we would like to
/// execute against a running Tendermint node.
#[derive(Debug)]
pub struct Plan {
    // Where to store all the raw JSON requests we generate.
    requests_path: PathBuf,
    // Where to store all the raw JSON responses we get back.
    responses_path: PathBuf,
    // Default period to wait before executing a request, in milliseconds.
    request_wait: Duration,
    // The list of requests we would like to execute (one at a time, in sequence).
    requests: Vec<PlannedRequest>,
    // The client to use when executing the requests.
    client: WebSocketClient,
}

impl Plan {
    pub fn new(output_path: &Path, request_wait: Duration) -> Result<PlanBuilderWithPaths> {
        info!(
            "Saving request and response data to: {}",
            output_path.to_str().unwrap()
        );
        let requests_path = output_path.join("requests");
        let responses_path = output_path.join("responses");
        let paths = vec![&requests_path, &responses_path];
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
        Ok(PlanBuilderWithPaths {
            requests_path,
            responses_path,
            request_wait,
        })
    }

    pub async fn execute(&mut self) -> Result<()> {
        info!("Executing plan");
        for request in &self.requests {
            request.write(&self.requests_path).await?;
            let response = match request.execute(&mut self.client, self.request_wait).await {
                Ok(r) => r,
                Err(e) => match e {
                    Error::Failed(_, r) => r,
                    _ => return Err(e),
                },
            };
            let response_output = self.responses_path.join(format!("{}.json", request.name));
            tokio::fs::write(
                &response_output,
                serde_json::to_string_pretty(&response).unwrap(),
            )
            .await?;
            info!(
                "Executed \"{}\". Wrote response to {}",
                request.name,
                response_output.to_str().unwrap()
            );
        }
        Ok(())
    }

    pub async fn execute_and_close(mut self) -> Result<()> {
        self.execute().await?;
        self.client.close().await
    }
}

pub struct PlanBuilderWithPaths {
    requests_path: PathBuf,
    responses_path: PathBuf,
    request_wait: Duration,
}

impl PlanBuilderWithPaths {
    pub fn then(self, request: impl Into<PlannedRequest>) -> PlanBuilderWithRequests {
        PlanBuilderWithRequests {
            requests_path: self.requests_path,
            responses_path: self.responses_path,
            request_wait: self.request_wait,
            requests: vec![request.into()],
        }
    }
}

pub struct PlanBuilderWithRequests {
    requests_path: PathBuf,
    responses_path: PathBuf,
    request_wait: Duration,
    requests: Vec<PlannedRequest>,
}

impl PlanBuilderWithRequests {
    pub fn then(mut self, request: impl Into<PlannedRequest>) -> PlanBuilderWithRequests {
        self.requests.push(request.into());
        self
    }

    /// Attempts to connect to the given WebSocket address and finally
    /// construct the request plan.
    ///
    /// On success, this also returns the WebSocket client driver, such that
    /// the caller can execute the driver.
    pub async fn connect(self, addr: &str) -> Result<(Plan, WebSocketClientDriver)> {
        let (client, driver) = WebSocketClient::new(addr).await?;
        Ok((
            Plan {
                requests_path: self.requests_path,
                responses_path: self.responses_path,
                requests: self.requests,
                request_wait: self.request_wait,
                client,
            },
            driver,
        ))
    }
}

/// A request with additional information attached to it pertaining to its
/// execution within its plan.
#[derive(Debug)]
pub struct PlannedRequest {
    // What do we call this planned request?
    name: String,
    // The request to be executed.
    request: Request,
    // The minimum height at which this request can be executed.
    min_height: Option<u64>,
    // Custom wait period before executing this request.
    wait: Option<Duration>,
}

impl PlannedRequest {
    pub fn new(name: &str, request: Request) -> Self {
        Self {
            request,
            name: name.to_owned(),
            min_height: None,
            wait: None,
        }
    }

    pub fn with_min_height(mut self, h: u64) -> Self {
        self.min_height = Some(h);
        self
    }

    /// Dumps the full wrapped JSON request into the given output directory.
    ///
    /// This planned request's name is used as the filename, and extension is
    /// `.json`.
    pub async fn write(&self, output_dir: &Path) -> Result<()> {
        let output_file = output_dir.join(format!("{}.json", self.name));
        tokio::fs::write(&output_file, self.request.to_string()).await?;
        debug!(
            "Wrote request \"{}\" to: {}",
            self.name,
            output_file.to_str().unwrap()
        );
        Ok(())
    }

    /// Execute this planned request using the given client.
    pub async fn execute(
        &self,
        client: &mut WebSocketClient,
        default_wait: Duration,
    ) -> Result<serde_json::Value> {
        info!("Executing request \"{}\"...", self.name);
        if let Some(min_height) = self.min_height {
            info!(" - Waiting for height {}...", min_height);
            self.wait_for_height(client, min_height).await?;
            info!(" - Target height reached");
        }
        let wait = self.wait.unwrap_or(default_wait);
        tokio::time::sleep(wait).await;
        self.request.execute(client).await
    }

    async fn wait_for_height(&self, client: &mut WebSocketClient, h: u64) -> Result<()> {
        let mut subs = client
            .subscribe(uuid_v4(), "tm.event = 'NewBlock'".to_owned())
            .await?;
        while let Some(result) = subs.next().await {
            let resp = result?;
            // TODO(thane): Find a more readable way of getting this value.
            let height = resp
                .get("result")
                .unwrap()
                .get("data")
                .unwrap()
                .get("value")
                .unwrap()
                .get("block")
                .unwrap()
                .get("header")
                .unwrap()
                .get("height")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
                .parse::<u64>()
                .unwrap();
            if height >= h {
                return Ok(());
            }
        }
        Err(Error::InternalError(format!(
            "subscription terminated before we could reach target height of {}",
            h
        )))
    }
}

impl From<Request> for PlannedRequest {
    fn from(request: Request) -> Self {
        let name = request.method.clone();
        Self {
            request,
            name,
            min_height: None,
            wait: None,
        }
    }
}
