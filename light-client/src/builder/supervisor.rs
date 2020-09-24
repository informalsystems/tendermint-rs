use tendermint::net;

use crate::builder::error::Error;
use crate::evidence::ProdEvidenceReporter;
use crate::fork_detector::ProdForkDetector;
use crate::peer_list::{PeerList, PeerListBuilder};
use crate::supervisor::{Instance, Supervisor};
use crate::types::PeerId;

pub struct Init;
pub struct HasPrimary;
pub struct Done;

/// Builder for the [`Supervisor`]
#[must_use]
pub struct SupervisorBuilder<State> {
    instances: PeerListBuilder<Instance>,
    addresses: PeerListBuilder<net::Address>,
    #[allow(dead_code)]
    state: State,
}

impl<Current> SupervisorBuilder<Current> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> SupervisorBuilder<Next> {
        SupervisorBuilder {
            instances: self.instances,
            addresses: self.addresses,
            state,
        }
    }
}

impl Default for SupervisorBuilder<Init> {
    fn default() -> Self {
        Self::new()
    }
}

impl SupervisorBuilder<Init> {
    /// Create an empty builder
    pub fn new() -> Self {
        Self {
            instances: PeerListBuilder::default(),
            addresses: PeerListBuilder::default(),
            state: Init,
        }
    }

    /// Set the primary [`Instance`].
    pub fn primary(
        mut self,
        peer_id: PeerId,
        address: net::Address,
        instance: Instance,
    ) -> Result<SupervisorBuilder<HasPrimary>, Error> {
        self.instances = self.instances.primary(peer_id, instance);
        self.addresses = self.addresses.primary(peer_id, address);

        Ok(self.with_state(HasPrimary))
    }
}

impl SupervisorBuilder<HasPrimary> {
    /// Add a witness [`Instance`].
    pub fn witness(
        mut self,
        peer_id: PeerId,
        address: net::Address,
        instance: Instance,
    ) -> Result<SupervisorBuilder<Done>, Error> {
        self.instances = self.instances.witness(peer_id, instance);
        self.addresses = self.addresses.witness(peer_id, address);

        Ok(self.with_state(Done))
    }
}

impl SupervisorBuilder<Done> {
    /// Build a production (non-mock) [`Supervisor`].
    #[must_use]
    pub fn build_prod(self) -> Supervisor {
        let (instances, addresses) = self.unwrap();

        Supervisor::new(
            instances,
            ProdForkDetector::default(),
            ProdEvidenceReporter::new(addresses.into_values()),
        )
    }

    /// Get the underlying list of instances and addresses.
    #[must_use]
    pub fn unwrap(self) -> (PeerList<Instance>, PeerList<net::Address>) {
        (self.instances.build(), self.addresses.build())
    }
}
