//! Testing-related utilities for the Tendermint RPC Client. Here we aim to
//! provide some useful abstractions in cases where you may want to use an RPC
//! client in your code, but mock its remote endpoint(s).

use async_trait::async_trait;
use std::{collections::HashMap, path::Path};
use tokio::fs;

use crate::{client::transport::Transport, Error};

/// A rudimentary fixture-based transport, where certain requests are
/// preconfigured to always produce specific kinds of responses.
#[derive(Debug)]
pub struct MappedFixtureTransport {
    successes: HashMap<String, String>,
    failures: HashMap<String, Error>,
}

#[async_trait]
impl Transport for MappedFixtureTransport {
    async fn request(&self, request: String) -> Result<String, Error> {
        match self.successes.get(&request) {
            Some(response) => Ok(response.clone()),
            None => match self.failures.get(&request) {
                Some(e) => Err(e.clone()),
                None => Err(Error::internal_error(
                    "no request/response mapping for supplied request",
                )),
            },
        }
    }
}

impl MappedFixtureTransport {
    /// Instantiate a new, empty mapped fixture transport (all requests will
    /// generate errors).
    pub fn new() -> Self {
        Self {
            successes: HashMap::new(),
            failures: HashMap::new(),
        }
    }

    /// Reads a JSON fixture for a request and response from the given
    /// filesystem paths.
    pub async fn read_success_fixture(
        &mut self,
        request_path: &Path,
        response_path: &Path,
    ) -> &mut Self {
        self.successes.insert(
            fs::read_to_string(request_path).await.unwrap(),
            fs::read_to_string(response_path).await.unwrap(),
        );
        self
    }

    /// Reads a JSON fixture for a request and maps it to the given error.
    pub async fn read_failure_fixture(&mut self, request_path: &Path, err: Error) -> &mut Self {
        self.failures
            .insert(fs::read_to_string(request_path).await.unwrap(), err);
        self
    }
}
