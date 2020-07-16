//! LightNode Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use abscissa_core::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

use crate::config::LightStoreConfig::OnDisk;
use tendermint_light_client::light_client;
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

    /// RPC related config parameters.
    pub rpc_config: RpcConfig,

    // TODO "now" should probably always be passed in as `Time::now()`
    /// The actual light client instances' configuration.
    /// Note: the first config will be used in the subjectively initialize
    /// the light node in the `initialize` subcommand.
    pub light_clients: Vec<LightClientConfig>,

    /// Shared state for all the LightClients.
    pub shared_state_config: LightStoreConfig,
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
    /// Configuration for LightClient state.
    pub state: LightStoreConfig,
}

/// LightClientStateConfig contains options for state type along with other configuration options.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LightStoreConfig {
    InMemory,
    OnDisk { db_path: PathBuf },
}

/// RpcConfig contains for the RPC server of the light node as
/// well as RPC client related options.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    /// The address the RPC server will serve.
    pub listen_addr: SocketAddr,
    /// The duration after which any RPC request to tendermint node will time out.
    pub request_timeout: Duration,
}

/// Default light client config settings.
impl Default for LightClientConfig {
    fn default() -> Self {
        Self {
            address: "tcp://127.0.0.1:26657".parse().unwrap(),
            peer_id: "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap(),
            state: OnDisk {
                db_path: "./lightstore/BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE"
                    .parse()
                    .unwrap(),
            },
        }
    }
}

/// Default configuration settings.
impl Default for LightNodeConfig {
    fn default() -> Self {
        Self {
            trusting_period: Duration::from_secs(864_000), // 60*60*24*10
            trust_threshold: TrustThreshold {
                numerator: 1,
                denominator: 3,
            },
            clock_drift: Duration::from_secs(1),
            rpc_config: RpcConfig {
                listen_addr: "127.0.0.1:8888".parse().unwrap(),
                request_timeout: Duration::from_secs(60),
            },
            // TODO(ismail): need at least 2 peers for a proper init
            // otherwise the light node will complain on `start` with `no witness left`
            light_clients: vec![LightClientConfig::default()],
            shared_state_config: OnDisk {
                db_path: "./lightstore".parse().unwrap(),
            },
        }
    }
}

impl From<LightNodeConfig> for light_client::Options {
    fn from(lnc: LightNodeConfig) -> Self {
        Self {
            trust_threshold: lnc.trust_threshold,
            trusting_period: lnc.trusting_period,
            clock_drift: lnc.clock_drift,
        }
    }
}
