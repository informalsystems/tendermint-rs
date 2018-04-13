//! Configuration for the `YubiHSM` backend

use std::collections::BTreeMap;

/// Configuration for a particular yubihsm-connector process
#[derive(Deserialize, Debug)]
pub struct YubihsmConnectorConfig {
    /// Address of yubihsm-connector (IP or hostname)
    pub addr: String,

    /// Authentication key ID to use to authenticate to the YubiHSM
    #[serde(rename = "auth-key-id")]
    pub auth_key_id: u16,

    /// Password to use to authenticate to the YubiHSM
    // TODO: allow password to be read from an external password-file
    pub password: String,

    /// Map of labels to private key configurations
    pub keys: BTreeMap<String, YubihsmPrivateKey>,
}

#[derive(Deserialize, Debug)]
pub struct YubihsmPrivateKey {
    /// Signing key ID
    #[serde(rename = "key-id")]
    pub key_id: u16,
}
