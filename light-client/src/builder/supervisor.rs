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
    peers: PeerList<net::Address>,
    instances: PeerListBuilder<Instance>,
    #[allow(dead_code)]
    state: State,
}

impl<Current> SupervisorBuilder<Current> {
    fn with_state<Next>(self, state: Next) -> SupervisorBuilder<Next> {
        SupervisorBuilder {
            peers: self.peers,
            instances: self.instances,
            state,
        }
    }
}

impl SupervisorBuilder<Init> {
    /// TODO
    pub fn new(peers: PeerList<net::Address>) -> Self {
        Self {
            peers,
            instances: PeerListBuilder::default(),
            state: Init,
        }
    }

    /// TODO
    pub fn primary(
        mut self,
        make_instance: impl FnOnce(PeerId, PeerList<net::Address>) -> Result<Instance, Error>,
    ) -> Result<SupervisorBuilder<HasPrimary>, Error> {
        let primary_id = self.peers.primary_id();
        let instance = make_instance(primary_id, self.peers.clone())?;

        self.instances = self.instances.primary(primary_id, instance);

        Ok(self.with_state(HasPrimary))
    }
}

impl SupervisorBuilder<HasPrimary> {
    /// TODO
    pub fn witness(
        mut self,
        make_instance: impl Fn(PeerId, PeerList<net::Address>) -> Result<Instance, Error>,
    ) -> Result<SupervisorBuilder<Done>, Error> {
        for witness_id in self.peers.witnesses_ids().iter().copied() {
            let instance = make_instance(witness_id, self.peers.clone())?;
            self.instances = self.instances.witness(witness_id, instance);
        }

        Ok(self.with_state(Done))
    }
}

impl SupervisorBuilder<Done> {
    /// TODO
    #[must_use]
    pub fn build_prod(self) -> Supervisor {
        self.build_custom(|instances, peers| {
            Supervisor::new(
                instances,
                ProdForkDetector::default(),
                ProdEvidenceReporter::new(peers.into_values()),
            )
        })
    }

    /// TODO
    #[must_use]
    pub fn build_custom(
        self,
        build: impl FnOnce(PeerList<Instance>, PeerList<net::Address>) -> Supervisor,
    ) -> Supervisor {
        build(self.instances.build(), self.peers)
    }
}
