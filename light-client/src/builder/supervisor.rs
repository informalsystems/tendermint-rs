use std::time::Duration;

use crate::builder::error::Error;
use crate::light_client::LightClient;
use crate::peer_list::{PeerList, PeerListBuilder};
use crate::store::LightStore;
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
pub struct SupervisorBuilder<State, L, S> {
    instances: PeerListBuilder<Instance<L, S>>,
    addresses: PeerListBuilder<tendermint_rpc::Url>,
    evidence_reporting_timeout: Option<Duration>,
    #[allow(dead_code)]
    state: State,
}

impl<Current, L, S> SupervisorBuilder<Current, L, S> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> SupervisorBuilder<Next, L, S> {
        SupervisorBuilder {
            instances: self.instances,
            addresses: self.addresses,
            evidence_reporting_timeout: self.evidence_reporting_timeout,
            state,
        }
    }

    /// Set the timeout for fork evidence submission
    pub fn evidence_reporting_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.evidence_reporting_timeout = timeout;
        self
    }
}

impl<L, S> Default for SupervisorBuilder<Init, L, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L, S> SupervisorBuilder<Init, L, S> {
    /// Create an empty builder
    pub fn new() -> Self {
        Self {
            instances: PeerListBuilder::default(),
            addresses: PeerListBuilder::default(),
            evidence_reporting_timeout: None,
            state: Init,
        }
    }

    /// Set the primary [`Instance`].
    pub fn primary(
        mut self,
        peer_id: PeerId,
        address: tendermint_rpc::Url,
        instance: Instance<L, S>,
    ) -> SupervisorBuilder<HasPrimary, L, S> {
        self.instances.primary(peer_id, instance);
        self.addresses.primary(peer_id, address);

        self.with_state(HasPrimary)
    }
}

impl<L, S> SupervisorBuilder<HasPrimary, L, S> {
    /// Add a witness [`Instance`].
    pub fn witness(
        mut self,
        peer_id: PeerId,
        address: tendermint_rpc::Url,
        instance: Instance<L, S>,
    ) -> SupervisorBuilder<Done, L, S> {
        self.instances.witness(peer_id, instance);
        self.addresses.witness(peer_id, address);

        self.with_state(Done)
    }

    /// Add multiple witnesses at once.
    pub fn witnesses(
        mut self,
        witnesses: impl IntoIterator<Item = (PeerId, tendermint_rpc::Url, Instance<L, S>)>,
    ) -> Result<SupervisorBuilder<Done, L, S>, Error> {
        let mut iter = witnesses.into_iter().peekable();
        if iter.peek().is_none() {
            return Err(Error::empty_witness_list());
        }

        for (peer_id, address, instance) in iter {
            self.instances.witness(peer_id, instance);
            self.addresses.witness(peer_id, address);
        }

        Ok(self.with_state(Done))
    }
}

impl<L: LightClient, S: LightStore> SupervisorBuilder<Done, L, S> {
    /// Build a production (non-mock) [`Supervisor`].
    #[must_use]
    #[cfg(feature = "rpc-client")]
    pub fn build_prod(self) -> Supervisor<L, S, ProdForkDetector, ProdEvidenceReporter> {
        let timeout = self.evidence_reporting_timeout;
        let (instances, addresses) = self.inner();

        Supervisor::new(
            instances,
            ProdForkDetector::default(),
            ProdEvidenceReporter::new(addresses.into_values(), timeout),
        )
    }

    /// Get the underlying list of instances and addresses.
    #[must_use]
    pub fn inner(self) -> (PeerList<Instance<L, S>>, PeerList<tendermint_rpc::Url>) {
        (self.instances.build(), self.addresses.build())
    }
}
