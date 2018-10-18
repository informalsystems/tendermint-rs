//! The KMS makes outbound connections to the validator, and is technically a
//! client, however once connected it accepts incoming RPCs, and otherwise
//! acts as a service.
//!
//! To dance around the fact the KMS isn't actually a service, we refer to it
//! as a "Key Management System".

use signatory::{self, Decode, Ed25519Seed, Encode};
use signatory_dalek::Ed25519Signer;
use std::{
    panic,
    thread::{self, JoinHandle},
    time::Duration,
};

use config::{ConnectionConfig, SecretConnectionConfig, ValidatorConfig};
use ed25519::{PublicKey, SECRET_KEY_ENCODING};
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
        seccon,
        unix,
        reconnect,
    } = config;

    // Error out if the same validator has both seccon and unix configured
    if seccon.is_some() && unix.is_some() {
        error!(
            "a validator has {} and {} specified, can only chose one",
            seccon.unwrap().uri(),
            unix.unwrap().uri()
        );
        return;
    }

    // Error out if a validator doesn't specify any connection configuration
    if seccon.is_none() && unix.is_none() {
        error!("a validator has no connection configuration");
        return;
    }

    // Prepare connection config
    let conn_config = if seccon.is_some() {
        ConnectionConfig::SecretConnection(seccon.unwrap())
    } else {
        ConnectionConfig::UNIXConnection(unix.unwrap())
    };

    // Engage main I/O loop
    while let Err(e) = client_session(&conn_config) {
        error!("[{}] {}", conn_config.uri(), e);

        // Break out of the loop if auto-reconnect is explicitly disabled
        if reconnect {
            // TODO: configurable respawn delay
            thread::sleep(Duration::from_secs(RESPAWN_DELAY));
        } else {
            return;
        }
    }

    info!("[{}] session closed gracefully", conn_config.uri());
}

/// Establish a session with the validator and handle incoming requests
fn client_session(config: &ConnectionConfig) -> Result<(), Error> {
    panic::catch_unwind(move || {
        match config {
            // This validator will use a secret connection
            ConnectionConfig::SecretConnection(ref conf) => {
                // Load secret connection key
                match load_secret_connection_key(conf) {
                    Ok(key) => {
                        // Output secret connection node ID
                        log_kms_node_id(&key);

                        // Construct a secret connection session
                        let mut session = Session::new_seccon(&conf.addr, conf.port, &key)?;

                        info!("[{}] connected to validator successfully", conf.uri());

                        while session.handle_request()? {}
                    }

                    Err(e) => error!("couldn't load secret connection key: {}", e),
                }
            }

            // This validator will use a UNIX connection
            ConnectionConfig::UNIXConnection(ref conf) => {
                // Construct a UNIX connection session
                let mut session = Session::new_unix(&conf.socket_path)?;

                info!("[{}] waiting for a validator connection", conf.uri());

                while session.handle_request()? {}
            }
        };

        Ok(())
    }).unwrap_or_else(|e| Err(Error::from_panic(&e)))
}

/// Initialize KMS secret connection private key
fn load_secret_connection_key(config: &SecretConnectionConfig) -> Result<Ed25519Seed, Error> {
    let key_path = &config.secret_key_path;

    if key_path.exists() {
        Ok(Ed25519Seed::decode_from_file(key_path, SECRET_KEY_ENCODING)
            .map_err(|e| err!(ConfigError, "error loading {}: {}", key_path.display(), e))?)
    } else {
        let seed = Ed25519Seed::generate();
        seed.encode_to_file(key_path, SECRET_KEY_ENCODING)?;
        Ok(seed)
    }
}

/// Log the KMS node ID
fn log_kms_node_id(seed: &Ed25519Seed) {
    let public_key = PublicKey::from(signatory::public_key(&Ed25519Signer::from(seed)).unwrap());
    info!("KMS node ID: {}", &public_key);
}
