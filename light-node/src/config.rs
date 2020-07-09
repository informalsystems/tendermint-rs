//! LightNode Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use abscissa_core::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tendermint_light_client::types::{PeerId, TrustThreshold};

/// LightNode Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LightNodeConfig {
    /// The fraction of the total voting power of a known
    /// and trusted validator set is sufficient for a commit to be
    /// accepted going forward.
    pub trust_threshold: TrustThreshold,
    /// The duration until we consider a trusted state as expired.
    pub trusting_period: Duration,
    /// Correction parameter dealing with only approximately synchronized clocks.
    pub clock_drift: Duration,

    // TODO "now" should probably always be passed in as `Time::now()`
    /// The actual light client instances' configuration.
    /// Note: the first config will be used in the subjectively initialize
    /// the light node in the `initialize` subcommand.
    pub light_clients: Vec<LightClientConfig>,
}

/// LightClientConfig contains all options of a light client instance.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LightClientConfig {
    /// Address of the Tendermint fullnode to connect to and
    /// fetch LightBlock data from.
    pub address: tendermint::net::Address,
    /// PeerID of the same Tendermint fullnode.
    pub peer_id: PeerId,
    /// The data base folder for this instance's store.
    pub db_path: PathBuf,
}

/// Default light client config settings.
impl Default for LightClientConfig {
    fn default() -> Self {
        Self {
            address: "tcp://127.0.0.1:26657".parse().unwrap(),
            peer_id: "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap(),
            db_path: "./lightstore/BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE"
                .parse()
                .unwrap(),
        }
    }
}

/// Default configuration settings.
impl Default for LightNodeConfig {
    fn default() -> Self {
        Self {
            trusting_period: Duration::from_secs(51_840_000),
            trust_threshold: TrustThreshold {
                numerator: 1,
                denominator: 3,
            },
            clock_drift: Duration::from_secs(1),
            light_clients: vec![LightClientConfig::default()],
        }
    }
}
