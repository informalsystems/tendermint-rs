use core::time::Duration;

use tendermint_light_client_verifier::host_functions::CryptoProvider;
#[cfg(feature = "rpc-client")]
use {
    crate::evidence::ProdEvidenceReporter, crate::fork_detector::ProdForkDetector,
    crate::supervisor::Supervisor,
};

use crate::{
    builder::error::Error,
    peer_list::{PeerList, PeerListBuilder},
    supervisor::Instance,
    verifier::types::PeerId,
};

pub struct Init;
pub struct HasPrimary;
pub struct Done;

/// Builder for the [`Supervisor`]
#[must_use]
pub struct SupervisorBuilder<State, HostFunctions> {
    instances: PeerListBuilder<Instance<HostFunctions>>,
    addresses: PeerListBuilder<tendermint_rpc::Url>,
    evidence_reporting_timeout: Option<Duration>,
    #[allow(dead_code)]
    state: State,
}

impl<Current, HostFunctions> SupervisorBuilder<Current, HostFunctions> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> SupervisorBuilder<Next, HostFunctions> {
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

impl<HostFunctions> Default for SupervisorBuilder<Init, HostFunctions> {
    fn default() -> Self {
        Self::new()
    }
}

impl<HostFunctions> SupervisorBuilder<Init, HostFunctions> {
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
        instance: Instance<HostFunctions>,
    ) -> SupervisorBuilder<HasPrimary, HostFunctions> {
        self.instances.primary(peer_id, instance);
        self.addresses.primary(peer_id, address);

        self.with_state(HasPrimary)
    }
}

impl<HostFunctions> SupervisorBuilder<HasPrimary, HostFunctions> {
    /// Add a witness [`Instance`].
    pub fn witness(
        mut self,
        peer_id: PeerId,
        address: tendermint_rpc::Url,
        instance: Instance<HostFunctions>,
    ) -> SupervisorBuilder<Done, HostFunctions> {
        self.instances.witness(peer_id, instance);
        self.addresses.witness(peer_id, address);

        self.with_state(Done)
    }

    /// Add multiple witnesses at once.
    pub fn witnesses(
        mut self,
        witnesses: impl IntoIterator<Item = (PeerId, tendermint_rpc::Url, Instance<HostFunctions>)>,
    ) -> Result<SupervisorBuilder<Done, HostFunctions>, Error> {
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

impl<HostFunctions> SupervisorBuilder<Done, HostFunctions>
where
    HostFunctions: CryptoProvider,
{
    /// Build a production (non-mock) [`Supervisor`].
    #[must_use]
    #[cfg(feature = "rpc-client")]
    pub fn build_prod(self) -> Supervisor<HostFunctions> {
        let timeout = self.evidence_reporting_timeout;
        let (instances, addresses) = self.inner();

        Supervisor::new(
            instances,
            ProdForkDetector::<HostFunctions>::default(),
            ProdEvidenceReporter::new(addresses.into_values(), timeout),
        )
    }

    /// Get the underlying list of instances and addresses.
    #[must_use]
    pub fn inner(
        self,
    ) -> (
        PeerList<Instance<HostFunctions>>,
        PeerList<tendermint_rpc::Url>,
    ) {
        (self.instances.build(), self.addresses.build())
    }
}
