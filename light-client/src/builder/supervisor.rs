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

/// TODO
#[must_use]
pub struct SupervisorBuilder<State> {
    instances: PeerListBuilder<Instance>,
    addresses: PeerListBuilder<net::Address>,
    #[allow(dead_code)]
    state: State,
}

impl<Current> SupervisorBuilder<Current> {
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
    /// TODO
    pub fn new() -> Self {
        Self {
            instances: PeerListBuilder::default(),
            addresses: PeerListBuilder::default(),
            state: Init,
        }
    }

    /// TODO
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
    /// TODO
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
    /// TODO
    #[must_use]
    pub fn build_prod(self) -> Supervisor {
        self.build_custom(|instances, addresses| {
            Supervisor::new(
                instances,
                ProdForkDetector::default(),
                ProdEvidenceReporter::new(addresses.into_values()),
            )
        })
    }

    /// TODO
    #[must_use]
    pub fn build_custom(
        self,
        build: impl FnOnce(PeerList<Instance>, PeerList<net::Address>) -> Supervisor,
    ) -> Supervisor {
        build(self.instances.build(), self.addresses.build())
    }
}
