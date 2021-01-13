//! Trivial ABCI application that just implements echo functionality.

use crate::Application;
use tendermint::abci::{request, response};

/// Trivial ABCI application that just implements echo functionality.
#[derive(Clone)]
pub struct EchoApp {
    data: String,
    version: String,
    app_version: u64,
}

impl EchoApp {
    /// Constructor.
    pub fn new<S: AsRef<str>>(data: S, version: S, app_version: u64) -> Self {
        Self {
            data: data.as_ref().to_owned(),
            version: version.as_ref().to_owned(),
            app_version,
        }
    }
}

impl Default for EchoApp {
    fn default() -> Self {
        EchoApp::new("Echo App", "0.0.1", 1)
    }
}

impl Application for EchoApp {
    fn info(&self, _req: request::Info) -> response::Info {
        response::Info::new(&self.data, &self.version, self.app_version, 1, [])
    }
}
