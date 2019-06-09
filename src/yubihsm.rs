//! Application-local YubiHSM configuration and initialization

use crate::{
    config::provider::yubihsm::YubihsmConfig,
    error::{Error, ErrorKind},
    prelude::*,
};
use lazy_static::lazy_static;
use std::{
    process,
    sync::{Mutex, MutexGuard},
};
use yubihsm::{Client, Connector};
#[cfg(not(feature = "yubihsm-mock"))]
use {
    crate::config::provider::yubihsm::AdapterConfig,
    yubihsm::{device::SerialNumber, UsbConfig},
};

lazy_static! {
    /// Connection to the YubiHSM device
    static ref HSM_CONNECTOR: Connector = init_connector();

    /// Authenticated client connection to the YubiHSM device
    static ref HSM_CLIENT: Mutex<Client> = Mutex::new(init_client());
}

/// Get the global HSM connector configured from global settings
pub fn connector() -> &'static Connector {
    &HSM_CONNECTOR
}

/// Get an HSM client configured from global settings
pub fn client() -> MutexGuard<'static, Client> {
    HSM_CLIENT.lock().unwrap()
}

/// Open a session with the YubiHSM2 using settings from the global config
#[cfg(not(feature = "yubihsm-mock"))]
fn init_connector() -> Connector {
    let cfg = config();

    match cfg.adapter {
        AdapterConfig::Http { ref connector } => Connector::http(connector),
        AdapterConfig::Usb { timeout_ms } => {
            let usb_config = UsbConfig {
                serial: cfg
                    .serial_number
                    .as_ref()
                    .map(|serial| serial.parse::<SerialNumber>().unwrap()),
                timeout_ms,
            };

            Connector::usb(&usb_config)
        }
    }
}

#[cfg(feature = "yubihsm-mock")]
fn init_connector() -> Connector {
    Connector::mockhsm()
}

/// Get a `yubihsm::Client` configured from the global configuration
fn init_client() -> Client {
    let credentials = config().auth.credentials();

    Client::open(connector().clone(), credentials, true).unwrap_or_else(|e| {
        status_err!("error connecting to YubiHSM2: {}", e);
        process::exit(1);
    })
}

/// Get the YubiHSM-related configuration
pub fn config() -> YubihsmConfig {
    let kms_config = app_config();
    let yubihsm_configs = &kms_config.providers.yubihsm;

    if yubihsm_configs.len() != 1 {
        status_err!(
            "expected one [yubihsm.provider] in config, found: {}",
            yubihsm_configs.len()
        );
        process::exit(1);
    }

    yubihsm_configs[0].clone()
}

impl From<yubihsm::client::Error> for Error {
    fn from(other: yubihsm::client::Error) -> Error {
        abscissa::Error::new(ErrorKind::from(other.kind()), Some(other.to_string())).into()
    }
}

impl From<yubihsm::connector::ErrorKind> for ErrorKind {
    fn from(other: yubihsm::connector::ErrorKind) -> ErrorKind {
        match other {
            yubihsm::connector::ErrorKind::AddrInvalid => ErrorKind::ConfigError,
            yubihsm::connector::ErrorKind::AccessDenied => ErrorKind::AccessError,
            yubihsm::connector::ErrorKind::IoError
            | yubihsm::connector::ErrorKind::ConnectionFailed
            | yubihsm::connector::ErrorKind::DeviceBusyError
            | yubihsm::connector::ErrorKind::RequestError
            | yubihsm::connector::ErrorKind::ResponseError
            | yubihsm::connector::ErrorKind::UsbError => ErrorKind::IoError,
        }
    }
}

impl From<yubihsm::client::ErrorKind> for ErrorKind {
    fn from(other: yubihsm::client::ErrorKind) -> ErrorKind {
        match other {
            yubihsm::client::ErrorKind::AuthenticationError => ErrorKind::AccessError,
            yubihsm::client::ErrorKind::ConnectorError { kind } => kind.into(),
            yubihsm::client::ErrorKind::DeviceError { kind } => kind.into(),
            yubihsm::client::ErrorKind::CreateFailed
            | yubihsm::client::ErrorKind::ProtocolError
            | yubihsm::client::ErrorKind::ClosedSessionError
            | yubihsm::client::ErrorKind::ResponseError => ErrorKind::IoError,
        }
    }
}

impl From<yubihsm::device::ErrorKind> for ErrorKind {
    fn from(other: yubihsm::device::ErrorKind) -> ErrorKind {
        // TODO(tarcieri): better map these to approriate KMS errors
        match other {
            yubihsm::device::ErrorKind::AuthenticationFailed => ErrorKind::AccessError,
            yubihsm::device::ErrorKind::InvalidCommand
            | yubihsm::device::ErrorKind::InvalidData
            | yubihsm::device::ErrorKind::InvalidSession
            | yubihsm::device::ErrorKind::SessionsFull
            | yubihsm::device::ErrorKind::SessionFailed
            | yubihsm::device::ErrorKind::StorageFailed
            | yubihsm::device::ErrorKind::WrongLength
            | yubihsm::device::ErrorKind::InsufficientPermissions
            | yubihsm::device::ErrorKind::LogFull
            | yubihsm::device::ErrorKind::ObjectNotFound
            | yubihsm::device::ErrorKind::InvalidId
            | yubihsm::device::ErrorKind::InvalidOtp
            | yubihsm::device::ErrorKind::GenericError
            | yubihsm::device::ErrorKind::ObjectExists => ErrorKind::SigningError,
            _ => ErrorKind::SigningError,
        }
    }
}
