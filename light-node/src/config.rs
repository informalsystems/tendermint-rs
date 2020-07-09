//! LightNode Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tendermint_light_client::types::{PeerId, TrustThreshold};
use abscissa_core::path::PathBuf;

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

/// Default configuration settings.
impl Default for LightNodeConfig {
    fn default() -> Self {
        Self {
            rpc_address: "localhost:26657".to_owned(),
            trusting_period: Duration::new(6000, 0),
            subjective_init: SubjectiveInit::default(),
        }
    }
}

/// Configuration for subjective initialization.
///
/// Contains the subjective height and validators hash (as a string formatted as hex).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectiveInit {
    /// Subjective height.
    pub height: u64,
    /// Subjective validators hash.
    pub validators_hash: String,
}

impl Default for SubjectiveInit {
    fn default() -> Self {
        Self {
            height: 1,
            // TODO(liamsi): a default hash here does not make sense unless it is a valid hash
            // from a public network
            validators_hash: "A5A7DEA707ADE6156F8A981777CA093F178FC790475F6EC659B6617E704871DD"
                .to_owned(),
        }
    }
}
