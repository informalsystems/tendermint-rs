use crate::{
    config::{provider::yubihsm::YubihsmConfig, KmsConfig},
    error::{KmsError, KmsErrorKind},
};
use abscissa::{Error, GlobalConfig};
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
    static ref HSM_CONNECTOR: Connector = init_connector();
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
    let kms_config = KmsConfig::get_global();
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

impl From<yubihsm::ClientError> for KmsError {
    fn from(other: yubihsm::ClientError) -> KmsError {
        Error::new(KmsErrorKind::from(other.kind()), Some(other.to_string())).into()
    }
}

impl From<yubihsm::connector::ConnectionErrorKind> for KmsErrorKind {
    fn from(other: yubihsm::connector::ConnectionErrorKind) -> KmsErrorKind {
        use yubihsm::connector::ConnectionErrorKind;

        match other {
            ConnectionErrorKind::AddrInvalid => KmsErrorKind::ConfigError,
            ConnectionErrorKind::AccessDenied => KmsErrorKind::AccessError,
            ConnectionErrorKind::IoError
            | ConnectionErrorKind::ConnectionFailed
            | ConnectionErrorKind::DeviceBusyError
            | ConnectionErrorKind::RequestError
            | ConnectionErrorKind::ResponseError
            | ConnectionErrorKind::UsbError => KmsErrorKind::IoError,
        }
    }
}

impl From<yubihsm::client::ClientErrorKind> for KmsErrorKind {
    fn from(other: yubihsm::client::ClientErrorKind) -> KmsErrorKind {
        use yubihsm::client::ClientErrorKind;

        match other {
            ClientErrorKind::AuthenticationError => KmsErrorKind::AccessError,
            ClientErrorKind::ConnectionError { kind } => kind.into(),
            ClientErrorKind::DeviceError { kind } => kind.into(),
            ClientErrorKind::CreateFailed
            | ClientErrorKind::ProtocolError
            | ClientErrorKind::ClosedSessionError
            | ClientErrorKind::ResponseError => KmsErrorKind::IoError,
        }
    }
}

impl From<yubihsm::DeviceErrorKind> for KmsErrorKind {
    fn from(other: yubihsm::device::DeviceErrorKind) -> KmsErrorKind {
        use yubihsm::device::DeviceErrorKind;

        // TODO(tarcieri): better map these to approriate KMS errors
        match other {
            DeviceErrorKind::AuthenticationFailed => KmsErrorKind::AccessError,
            DeviceErrorKind::InvalidCommand
            | DeviceErrorKind::InvalidData
            | DeviceErrorKind::InvalidSession
            | DeviceErrorKind::SessionsFull
            | DeviceErrorKind::SessionFailed
            | DeviceErrorKind::StorageFailed
            | DeviceErrorKind::WrongLength
            | DeviceErrorKind::InsufficientPermissions
            | DeviceErrorKind::LogFull
            | DeviceErrorKind::ObjectNotFound
            | DeviceErrorKind::InvalidId
            | DeviceErrorKind::InvalidOtp
            | DeviceErrorKind::GenericError
            | DeviceErrorKind::ObjectExists => KmsErrorKind::SigningError,
            _ => KmsErrorKind::SigningError,
        }
    }
}
