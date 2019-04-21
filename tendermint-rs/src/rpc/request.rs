//! JSONRPC requests

use crate::Error;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// JSONRPC requests
pub trait Request {
    /// Response type for this command
    type Response: super::response::Response;

    /// Path for this request
    fn path(&self) -> Path;
}

/// JSONRPC request paths
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Path(String);

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Path {
    type Err = Error;

    /// Parse a request path from a string
    fn from_str(path: &str) -> Result<Self, Error> {
        Ok(Path(path.to_owned()))
    }
}
