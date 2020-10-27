use std::time::Duration;

use tendermint::net;

use crate::builder::error::{self, Error};
use crate::peer_list::PeerListBuilder;
use crate::supervisor::Instance;

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
    instances: PeerListBuilder<net::Address, Instance>,
    evidence_reporting_timeout: Option<Duration>,
    #[allow(dead_code)]
    state: State,
}

impl<Current> SupervisorBuilder<Current> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> SupervisorBuilder<Next> {
        SupervisorBuilder {
            instances: self.instances,
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
            evidence_reporting_timeout: None,
            state: Init,
        }
    }

    /// Set the primary [`Instance`].
    pub fn primary(
        mut self,
        address: net::Address,
        instance: Instance,
    ) -> SupervisorBuilder<HasPrimary> {
        self.instances.primary(address, instance);

        self.with_state(HasPrimary)
    }
}

impl SupervisorBuilder<HasPrimary> {
    /// Add a witness [`Instance`].
    pub fn witness(mut self, address: net::Address, instance: Instance) -> SupervisorBuilder<Done> {
        self.instances.witness(address, instance);

        self.with_state(Done)
    }

    /// Add multiple witnesses at once.
    pub fn witnesses(
        mut self,
        witnesses: impl IntoIterator<Item = (net::Address, Instance)>,
    ) -> Result<SupervisorBuilder<Done>, Error> {
        let mut iter = witnesses.into_iter().peekable();
        if iter.peek().is_none() {
            return Err(error::Kind::EmptyWitnessList.into());
        }

        for (address, instance) in iter {
            self.instances.witness(address, instance);
        }

        Ok(self.with_state(Done))
    }
}

impl SupervisorBuilder<Done> {
    /// Build a production (non-mock) [`Supervisor`].
    #[must_use]
    #[cfg(feature = "rpc-client")]
    pub fn build_prod(self) -> Supervisor {
        let timeout = self.evidence_reporting_timeout;

        Supervisor::new(
            self.instances.build(),
            ProdForkDetector::default(),
            ProdEvidenceReporter::new(timeout),
        )
    }
}
