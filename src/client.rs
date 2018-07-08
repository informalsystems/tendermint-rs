//! The KMS makes outbound connections to the validator, and is technically a
//! client, however once connected it accepts incoming RPCs, and otherwise
//! acts as a service.
//!
//! To dance around the fact the KMS isn't actually a service, we refer to it
//! as a "Key Management System".

use std::panic;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use config::ValidatorConfig;
use ed25519::Keyring;
use failure::Error;
use session::Session;
use signatory::providers::dalek::Ed25519Signer as DalekSigner;


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
    pub fn spawn(label: String, config: ValidatorConfig, keyring: Arc<Keyring>, secret_connection_key: Arc<DalekSigner>) -> Self {
        Self {
            handle: thread::spawn(move || client_loop(&label, &config, &keyring, &secret_connection_key)),
        }
    }

    /// Wait for a running client to finish
    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

/// Main loop for all clients. Handles reconnecting in the event of an error
fn client_loop(label: &str, config: &ValidatorConfig, keyring: &Arc<Keyring>, secret_connection_key:&Arc<DalekSigner>) {
    let addr = &config.addr;
    let port = config.port;
    let info = format!("{} ({}:{})", label, addr, port);

    loop {
        match panic::catch_unwind(|| client_session(addr, port, keyring,secret_connection_key)) {
            Ok(result) => match result {
                Ok(_) => {
                    info!("[{}] session closed gracefully", &info);
                    return;
                }
                Err(e) => error!("[{}] {}", &info, e),
            },
            Err(val) => {
                if let Some(e) = val.downcast_ref::<String>() {
                    error!("[{}] client panic! {}", &info, e);
                } else if let Some(e) = val.downcast_ref::<&str>() {
                    error!("[{}] client panic! {}", &info, e);
                } else {
                    error!("[{}] client panic! (unknown cause)", &info);
                }
            }
        }

        // Break out of the loop if auto-reconnect is explicitly disabled
        if config.reconnect.is_some() && !config.reconnect.unwrap() {
            break;
        }

        // TODO: configurable respawn delay
        thread::sleep(Duration::from_secs(RESPAWN_DELAY))
    }
}

/// Establish a session with the validator and handle incoming requests
fn client_session(addr: &str, port: u16, keyring: &Arc<Keyring>,secret_connection_key: &Arc<DalekSigner> ) -> Result<(), Error> {
    let mut session = Session::new(addr, port, Arc::clone(keyring),Arc::clone(secret_connection_key))?;
    while session.handle_request()? {}
    Ok(())
}
