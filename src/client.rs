//! The KMS makes outbound connections to the validator, and is technically a
//! client, however once connected it accepts incoming RPCs, and otherwise
//! acts as a service.
//!
//! To dance around the fact the KMS isn't actually a service, we refer to it
//! as a "Key Management System".

use signatory::{self, Decode, Encode};
use signatory_dalek::Ed25519Signer;
use std::{
    panic,
    path::Path,
    thread::{self, JoinHandle},
    time::Duration,
};

use config::{ConnectionConfig, ValidatorConfig};
use ed25519::{self, SECRET_KEY_ENCODING};
use error::{KmsError, KmsErrorKind};
use session::Session;
use tendermint::chain;

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
    pub fn spawn(config: ValidatorConfig) -> Self {
        Self {
            handle: thread::spawn(move || client_loop(config)),
        }
    }

    /// Wait for a running client to finish
    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

/// Main loop for all clients. Handles reconnecting in the event of an error
fn client_loop(config: ValidatorConfig) {
    let ValidatorConfig {
        chain_id,
        connection,
        reconnect,
    } = config;

    while let Err(e) = client_session(chain_id, &connection) {
        error!("[{}] {}", connection.uri(), e);

        if reconnect {
            // TODO: configurable respawn delay
            thread::sleep(Duration::from_secs(RESPAWN_DELAY));
        } else {
            // Break out of the loop if auto-reconnect is explicitly disabled
            return;
        }
    }

    info!(
        "[{}@{}] session closed gracefully",
        chain_id,
        connection.uri()
    );
}

/// Establish a session with the validator and handle incoming requests
fn client_session(chain_id: chain::Id, config: &ConnectionConfig) -> Result<(), KmsError> {
    panic::catch_unwind(move || {
        match config {
            ConnectionConfig::Tcp {
                addr,
                port,
                secret_key_path,
            } => {
                let secret_key = load_secret_connection_key(secret_key_path)?;
                let node_public_key = ed25519::PublicKey::from(
                    signatory::public_key(&Ed25519Signer::from(&secret_key)).unwrap(),
                );

                info!("KMS node ID: {}", &node_public_key);

                let mut session = Session::connect_tcp(chain_id, addr, *port, &secret_key)?;

                info!(
                    "[{}@{}] connected to validator successfully",
                    chain_id,
                    config.uri()
                );

                session.request_loop()?;
            }
            ConnectionConfig::Unix { socket_path } => {
                // Construct a UNIX connection session
                let mut session = Session::accept_unix(chain_id, socket_path)?;

                info!(
                    "[{}@{}] waiting for a validator connection",
                    chain_id,
                    config.uri()
                );

                session.request_loop()?;
            }
        };

        Ok(())
    }).unwrap_or_else(|e| Err(KmsError::from_panic(&e)))
}

/// Initialize KMS secret connection private key
fn load_secret_connection_key(path: &Path) -> Result<ed25519::Seed, KmsError> {
    if path.exists() {
        Ok(
            ed25519::Seed::decode_from_file(path, SECRET_KEY_ENCODING).map_err(|e| {
                err!(
                    KmsErrorKind::ConfigError,
                    "error loading SecretConnection key from {}: {}",
                    path.display(),
                    e
                )
            })?,
        )
    } else {
        let seed = ed25519::Seed::generate();
        seed.encode_to_file(path, SECRET_KEY_ENCODING)?;
        Ok(seed)
    }
}
