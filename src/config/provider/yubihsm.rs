//! Configuration for the `YubiHSM` backend

use abscissa::{
    secrets::{BorrowSecret, DebugSecret, Secret},
    util::Zeroize,
};
use std::process;
use yubihsm::{Credentials, HttpConfig, SerialNumber, UsbConfig};

/// The (optional) `[providers.yubihsm]` config section
#[derive(Clone, Deserialize, Debug)]
pub struct YubihsmConfig {
    /// Adapter configuration
    pub adapter: AdapterConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// List of signing keys in this YubiHSM
    #[serde(default)]
    pub keys: Vec<SigningKeyConfig>,

    /// Serial number of the YubiHSM to connect to
    pub serial_number: Option<SerialNumber>,
}

impl YubihsmConfig {
    /// Get the `yubihsm::HttpConfig` or exit if unconfigured
    #[allow(dead_code)]
    pub fn http_config(&self) -> HttpConfig {
        match self.adapter {
            AdapterConfig::Http { ref connector } => connector.clone(),
            AdapterConfig::Usb { .. } => {
                status_err!("YubiHSM2 HTTP adapter support required, sorry");
                process::exit(1);
            }
        }
    }

    /// Get the `yubihsm::UsbConfig` or exit if unconfigured
    pub fn usb_config(&self) -> UsbConfig {
        match self.adapter {
            AdapterConfig::Http { .. } => {
                status_err!("YubiHSM2 USB adapter support required, sorry");
                process::exit(1);
            }
            AdapterConfig::Usb { timeout_ms } => UsbConfig {
                serial: self.serial_number,
                timeout_ms,
            },
        }
    }
}

/// Configuration for an individual YubiHSM
#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "type")]
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
        connector: HttpConfig,
    },
}

/// Configuration options for this connector
#[derive(Clone, Debug, Deserialize)]
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
        Credentials::from_password(self.key, self.password.borrow_secret().0.as_bytes())
    }
}

/// Password to the YubiHSM
#[derive(Clone, Deserialize)]
pub struct Password(String);

impl DebugSecret for Password {
    fn debug_secret(&self) -> &'static str {
        "REDACTED PASSWORD"
    }
}

impl Zeroize for Password {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SigningKeyConfig {
    /// Identifier for this key
    pub id: String,

    /// Signing key ID
    pub key: u16,
}

/// Default value for `AdapterConfig::Usb { timeout_ms }`
fn usb_timeout_ms_default() -> u64 {
    1000
}
