//! The KMS makes outbound connections to the validator, and is technically a
//! client, however once connected it accepts incoming RPCs, and otherwise
//! acts as a service.
//!
//! To dance around the fact the KMS isn't actually a service, we refer to it
//! as a "Key Management System".

use signatory::{ed25519, Decode, Encode};
use signatory_dalek::Ed25519Signer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{
    panic,
    path::Path,
    thread::{self, JoinHandle},
    time::Duration,
};
use tendermint::{chain, public_keys::SecretConnectionKey};

use crate::{
    config::{ValidatorAddr, ValidatorConfig},
    error::{KmsError, KmsErrorKind},
    keyring::SecretKeyEncoding,
    session::Session,
};

/// How long to wait after a crash before respawning (in seconds)
pub const RESPAWN_DELAY: u64 = 1;

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
    pub fn spawn(config: ValidatorConfig, should_term: Arc<AtomicBool>) -> Self {
        Self {
            handle: thread::spawn(move || client_loop(config, should_term)),
        }
    }

    /// Wait for a running client to finish
    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

/// Main loop for all clients. Handles reconnecting in the event of an error
fn client_loop(config: ValidatorConfig, should_term: Arc<AtomicBool>) {
    let ValidatorConfig {
        addr,
        chain_id,
        reconnect,
        secret_key,
    } = config;

    loop {
        let term = Arc::clone(&should_term);

        // If we've already received a shutdown signal from outside
        if term.load(Ordering::Relaxed) {
            info!("[{}@{}] shutdown request received", chain_id, &addr);
            return;
        }

        let session_result = match &addr {
            ValidatorAddr::Tcp { host, port } => match &secret_key {
                Some(path) => tcp_session(chain_id, host, *port, path, term),
                None => {
                    error!(
                        "config error: missing field `secret_key` for validator {}",
                        host
                    );
                    return;
                }
            },
            ValidatorAddr::Unix { socket_path } => unix_session(chain_id, socket_path, term),
        };

        if let Err(e) = session_result {
            error!("[{}@{}] {}", chain_id, addr, e);

            if reconnect {
                // TODO: configurable respawn delay
                thread::sleep(Duration::from_secs(RESPAWN_DELAY));
            } else {
                return;
            }
        } else {
            info!("[{}@{}] session closed gracefully", chain_id, &addr);
            // Indicate to the outer thread it's time to terminate
            Arc::clone(&should_term).swap(true, Ordering::Relaxed);

            return;
        }
    }
}

/// Create a TCP connection to a validator (encrypted with SecretConnection)
fn tcp_session(
    chain_id: chain::Id,
    host: &str,
    port: u16,
    secret_key_path: &Path,
    should_term: Arc<AtomicBool>,
) -> Result<(), KmsError> {
    let secret_key = load_secret_connection_key(secret_key_path)?;

    let node_public_key =
        SecretConnectionKey::from(ed25519::public_key(&Ed25519Signer::from(&secret_key)).unwrap());

    info!("KMS node ID: {}", &node_public_key);

    panic::catch_unwind(move || {
        let mut session = Session::connect_tcp(chain_id, host, port, &secret_key)?;

        info!(
            "[{}@tcp://{}:{}] connected to validator successfully",
            chain_id, host, port
        );

        session.request_loop(should_term)
    })
    .unwrap_or_else(|ref e| Err(KmsError::from_panic(e)))
}

/// Create a validator session over a Unix domain socket
fn unix_session(
    chain_id: chain::Id,
    socket_path: &Path,
    should_term: Arc<AtomicBool>,
) -> Result<(), KmsError> {
    panic::catch_unwind(move || {
        let mut session = Session::connect_unix(chain_id, socket_path)?;

        info!(
            "[{}@unix://{}] connected to validator successfully",
            chain_id,
            socket_path.display()
        );

        session.request_loop(should_term)
    })
    .unwrap_or_else(|ref e| Err(KmsError::from_panic(e)))
}

/// Initialize KMS secret connection private key
fn load_secret_connection_key(path: &Path) -> Result<ed25519::Seed, KmsError> {
    if path.exists() {
        Ok(
            ed25519::Seed::decode_from_file(path, &SecretKeyEncoding::default()).map_err(|e| {
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
        seed.encode_to_file(path, &SecretKeyEncoding::default())?;
        Ok(seed)
    }
}
