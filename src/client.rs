//! The KMS makes outbound connections to the validator, and is technically a
//! client, however once connected it accepts incoming RPCs, and otherwise
//! acts as a service.
//!
//! To dance around the fact the KMS isn't actually a service, we refer to it
//! as a "Key Management System".

use signatory::Ed25519Seed;
use std::{
    panic,
    thread::{self, JoinHandle},
    time::Duration,
};

use config::ValidatorConfig;
use error::Error;
use session::Session;

/// How long to wait after a crash before respawning (in seconds)
pub const RESPAWN_DELAY: u64 = 5;

/// Client connections: wraps a thread which makes a connection to a particular
/// validator node and then receives RPCs.
///
/// The `Client` type does not deal with network I/O, that is handled inside of
/// the `Session`. Instead, the `Client` type manages threading and respawning
/// sessions in the event of errors.
pub struct Client {
    /// Handle to the client thread
    handle: JoinHandle<()>,
}

impl Client {
    /// Spawn a new client, returning a handle so it can be joined
    pub fn spawn(config: ValidatorConfig, secret_connection_key: Ed25519Seed) -> Self {
        Self {
            handle: thread::spawn(move || client_loop(&config, &secret_connection_key)),
        }
    }

    /// Wait for a running client to finish
    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

/// Main loop for all clients. Handles reconnecting in the event of an error
fn client_loop(config: &ValidatorConfig, secret_connection_key: &Ed25519Seed) {
    while let Err(e) = client_session(&config.addr, config.port, secret_connection_key) {
        error!("[{}] {}", config.uri(), e);

        // Break out of the loop if auto-reconnect is explicitly disabled
        if config.reconnect {
            // TODO: configurable respawn delay
            thread::sleep(Duration::from_secs(RESPAWN_DELAY));
        } else {
            return;
        }
    }

    info!("[{}] session closed gracefully", config.uri());
}

/// Establish a session with the validator and handle incoming requests
fn client_session(addr: &str, port: u16, secret_connection_key: &Ed25519Seed) -> Result<(), Error> {
    panic::catch_unwind(move || {
        let mut session = Session::new(addr, port, &secret_connection_key)?;
        info!(
            "[gaia-rpc://{}:{}] connected to validator successfully",
            addr, port
        );
        while session.handle_request()? {}
        Ok(())
    }).unwrap_or_else(|e| Err(Error::from_panic(&e)))
}
