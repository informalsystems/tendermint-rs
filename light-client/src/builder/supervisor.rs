use tendermint::net;

use crate::builder::error::{self, Error};
use crate::peer_list::{PeerList, PeerListBuilder};
use crate::supervisor::Instance;
use crate::types::PeerId;

#[cfg(feature = "rpc-client")]
use {
    crate::evidence::ProdEvidenceReporter, crate::fork_detector::ProdForkDetector,
    crate::supervisor::Supervisor,
};

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
    ) -> SupervisorBuilder<HasPrimary> {
        self.instances.primary(peer_id, instance);
        self.addresses.primary(peer_id, address);

        self.with_state(HasPrimary)
    }
}

impl SupervisorBuilder<HasPrimary> {
    /// Add a witness [`Instance`].
    pub fn witness(
        mut self,
        peer_id: PeerId,
        address: net::Address,
        instance: Instance,
    ) -> SupervisorBuilder<Done> {
        self.instances.witness(peer_id, instance);
        self.addresses.witness(peer_id, address);

        self.with_state(Done)
    }

    /// Add multiple witnesses at once.
    pub fn witnesses(
        mut self,
        witnesses: impl IntoIterator<Item = (PeerId, net::Address, Instance)>,
    ) -> Result<SupervisorBuilder<Done>, Error> {
        let mut iter = witnesses.into_iter().peekable();
        if iter.peek().is_none() {
            return Err(error::Kind::EmptyWitnessList.into());
        }

        for (peer_id, address, instance) in iter {
            self.instances.witness(peer_id, instance);
            self.addresses.witness(peer_id, address);
        }

        Ok(self.with_state(Done))
    }
}

impl SupervisorBuilder<Done> {
    /// Build a production (non-mock) [`Supervisor`].
    #[must_use]
    #[cfg(feature = "rpc-client")]
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
