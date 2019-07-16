//! Configuration for the `YubiHSM` backend

use crate::chain;
use abscissa_core::secret::{CloneableSecret, DebugSecret, ExposeSecret, Secret};
use serde::Deserialize;
use tendermint::net;
use yubihsm::Credentials;
use zeroize::Zeroize;

/// The (optional) `[providers.yubihsm]` config section
#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct YubihsmConfig {
    /// Adapter configuration
    pub adapter: AdapterConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// List of signing keys in this YubiHSM
    #[serde(default)]
    pub keys: Vec<SigningKeyConfig>,

    /// Serial number of the YubiHSM to connect to
    pub serial_number: Option<String>,

    /// Configuration for `yubihsm-connector` compatible HTTP server.
    #[cfg(feature = "yubihsm-server")]
    pub connector_server: Option<ConnectorServerConfig>,
}

/// Configuration for an individual YubiHSM
#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum AdapterConfig {
    /// Connect to the YubiHSM2 directly via USB
    #[serde(rename = "usb")]
    Usb {
        /// Timeout when communicating with YubiHSM2
        #[serde(default = "usb_timeout_ms_default")]
        timeout_ms: u64,
    },

    /// Connect to the YubiHSM2 via `yubihsm-connector`
    #[serde(rename = "http")]
    Http {
        /// `yubihsm-connector` configuration
        addr: net::Address,
    },
}

/// Configuration options for this connector
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    /// Authentication key ID to use to authenticate to the YubiHSM
    pub key: u16,

    /// Password to use to authenticate to the YubiHSM
    // TODO: allow password to be read from an external password-file
    pub password: Secret<Password>,
}

impl AuthConfig {
    /// Get the `yubihsm::Credentials` for this `AuthConfig`
    pub fn credentials(&self) -> Credentials {
        Credentials::from_password(self.key, self.password.expose_secret().0.as_bytes())
    }
}

/// Password to the YubiHSM
#[derive(Clone, Deserialize, Zeroize)]
#[serde(deny_unknown_fields)]
#[zeroize(drop)]
pub struct Password(String);

impl CloneableSecret for Password {}

impl DebugSecret for Password {
    fn debug_secret() -> &'static str {
        "REDACTED PASSWORD"
    }
}

/// Signing key configuration
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningKeyConfig {
    /// Chains this signing key is authorized to be used from
    pub chain_ids: Vec<chain::Id>,

    /// Signing key ID
    pub key: u16,
}

/// Default value for `AdapterConfig::Usb { timeout_ms }`
fn usb_timeout_ms_default() -> u64 {
    1000
}

/// Configuration for `yubihsm-connector` compatible service
#[cfg(feature = "yubihsm-server")]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorServerConfig {
    /// Listen address to run the connector service at
    pub laddr: net::Address,

    /// Connect to the listen address when using `tmkms yubihsm` CLI
    pub cli: Option<CliConfig>,
}

/// Overrides for when using the `tmkms yubihsm` command-line interface
#[cfg(feature = "yubihsm-server")]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliConfig {
    /// Override the auth key to use when using the CLI. This will additionally
    /// prompt for a password from the terminal.
    pub auth_key: Option<u16>,
}
