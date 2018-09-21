#[cfg(feature = "softsign")]
pub mod softsign;
#[cfg(feature = "yubihsm")]
pub mod yubihsm;

#[cfg(feature = "softsign")]
use self::softsign::SoftSignConfig;
#[cfg(feature = "yubihsm")]
use self::yubihsm::YubihsmConfig;

/// Provider configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ProviderConfig {
    /// Software-backed signer
    #[cfg(feature = "softsign")]
    #[serde(default)]
    pub softsign: Vec<SoftSignConfig>,

    /// Map of yubihsm-connector labels to their configurations
    #[cfg(feature = "yubihsm")]
    #[serde(default)]
    pub yubihsm: Vec<YubihsmConfig>,
}
