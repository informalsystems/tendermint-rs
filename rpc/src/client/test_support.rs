//! Testing-related utilities for the Tendermint RPC Client. Here we aim to
//! provide some useful abstractions in cases where you may want to use an RPC
//! client in your code, but mock its remote endpoint's responses.

use std::path::PathBuf;
use tokio::fs;

pub mod matching_transport;

/// A fixture that can refer to a file in the filesystem or is a string in its
/// own right.
#[derive(Debug, Clone)]
pub enum Fixture {
    File(PathBuf),
    Raw(String),
}

impl Fixture {
    async fn read(&self) -> String {
        match self {
            Fixture::File(path) => fs::read_to_string(path.as_path()).await.unwrap(),
            Fixture::Raw(s) => s.clone(),
        }
    }
}

impl Into<Fixture> for String {
    fn into(self) -> Fixture {
        Fixture::Raw(self)
    }
}

impl Into<Fixture> for &str {
    fn into(self) -> Fixture {
        Fixture::Raw(self.to_string())
    }
}

impl Into<Fixture> for PathBuf {
    fn into(self) -> Fixture {
        Fixture::File(self)
    }
}
