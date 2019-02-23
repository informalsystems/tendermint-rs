use crate::config::{provider::yubihsm::YubihsmConfig, KmsConfig};
use abscissa::GlobalConfig;
use std::{
    process,
    sync::{Mutex, MutexGuard},
};
#[cfg(feature = "yubihsm-mock")]
use yubihsm::MockHsm;
#[cfg(not(feature = "yubihsm-mock"))]
use yubihsm::UsbConnector;
use yubihsm::{Client, Connector};

lazy_static! {
    static ref HSM_CLIENT: Mutex<Client> = Mutex::new(create_hsm_client());
}

/// Get a `Box<yubihsm::Connector>` instantiated via the configuration file
pub fn get_hsm_client() -> MutexGuard<'static, Client> {
    HSM_CLIENT.lock().unwrap()
}

/// Get a `yubihsm::Client` configured from the global configuration
fn create_hsm_client() -> Client {
    let connector = create_hsm_connector();
    let credentials = get_hsm_config().auth.credentials();

    Client::open(connector, credentials, true).unwrap_or_else(|e| {
        status_err!("error connecting to YubiHSM2: {}", e);
        process::exit(1);
    })
}

/// Get the YubiHSM-related configuration
pub fn get_hsm_config() -> YubihsmConfig {
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

/// Open a session with the YubiHSM2 using settings from the global config
#[cfg(not(feature = "yubihsm-mock"))]
pub fn create_hsm_connector() -> Box<dyn Connector> {
    // TODO: `HttpConnector` support
    let connector = UsbConnector::create(&get_hsm_config().usb_config()).unwrap_or_else(|e| {
        status_err!("error opening USB connection to YubiHSM2: {}", e);
        process::exit(1);
    });

    connector.into()
}

#[cfg(feature = "yubihsm-mock")]
pub fn create_hsm_connector() -> Box<dyn Connector> {
    MOCK_HSM.clone().into()
}

#[cfg(feature = "yubihsm-mock")]
lazy_static! {
    static ref MOCK_HSM: MockHsm = MockHsm::default();
}
