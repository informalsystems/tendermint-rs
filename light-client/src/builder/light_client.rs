//! DSL for building a light client

use std::time::Duration;

use tendermint::{block::Height, net, Hash};

use crate::bail;
use crate::builder::error::{self, Error};
use crate::components::clock::{Clock, SystemClock};
use crate::components::io::{AtHeight, Io, ProdIo};
use crate::components::scheduler::{self, Scheduler};
use crate::components::verifier::{ProdVerifier, Verifier};
use crate::light_client::{LightClient, Options};
use crate::operations::{Hasher, ProdHasher};
use crate::peer_list::PeerList;
use crate::state::{State, VerificationTrace};
use crate::store::LightStore;
use crate::supervisor::Instance;
use crate::types::{LightBlock, PeerId, Status};

/// TODO
pub struct NoTrustedState;

/// TODO
pub struct HasTrustedState;

/// Builder for light clients
#[must_use]
pub struct LightClientBuilder<State> {
    peer_id: PeerId,
    peers: PeerList<net::Address>,
    hasher: Box<dyn Hasher>,
    io: Box<dyn Io>,
    verifier: Box<dyn Verifier>,
    light_store: Box<dyn LightStore>,
    clock: Box<dyn Clock>,
    scheduler: Box<dyn Scheduler>,
    options: Options,
    #[allow(dead_code)]
    state: State,
}

impl<Current> LightClientBuilder<Current> {
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<Next> {
        LightClientBuilder {
            peer_id: self.peer_id,
            peers: self.peers,
            options: self.options,
            light_store: self.light_store,
            hasher: self.hasher,
            io: self.io,
            verifier: self.verifier,
            clock: self.clock,
            scheduler: self.scheduler,
            state,
        }
    }
}

impl LightClientBuilder<NoTrustedState> {
    /// TODO
    pub fn prod(
        peer_id: PeerId,
        peers: PeerList<net::Address>,
        light_store: Box<dyn LightStore>,
        options: Options,
        timeout: Option<Duration>,
    ) -> Self {
        let peer_map = peers.values().clone();

        Self::custom(
            peer_id,
            peers,
            options,
            light_store,
            Box::new(ProdHasher),
            Box::new(ProdIo::new(peer_map, timeout)),
            Box::new(ProdVerifier::default()),
            Box::new(SystemClock),
            Box::new(scheduler::basic_bisecting_schedule),
        )
    }
    /// TODO
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
        peers: PeerList<net::Address>,
        options: Options,
        light_store: Box<dyn LightStore>,
        hasher: Box<dyn Hasher>,
        io: Box<dyn Io>,
        verifier: Box<dyn Verifier>,
        clock: Box<dyn Clock>,
        scheduler: Box<dyn Scheduler>,
    ) -> Self {
        Self {
            peer_id,
            peers,
            hasher,
            io,
            verifier,
            light_store,
            clock,
            scheduler,
            options,
            state: NoTrustedState,
        }
    }

    /// TODO
    pub fn trust_light_block(
        mut self,
        trusted_state: LightBlock,
    ) -> LightClientBuilder<HasTrustedState> {
        self.light_store.insert(trusted_state, Status::Trusted);
        self.with_state(HasTrustedState)
    }

    /// TODO
    pub fn trust_primary_latest(mut self) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(self.peer_id, AtHeight::Highest)
            .map_err(error::Kind::Io)?;

        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// TODO
    pub fn trust_primary_at(
        mut self,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(self.peer_id, AtHeight::At(trusted_height))
            .map_err(error::Kind::Io)?;

        if trusted_state.height() != trusted_height {
            bail!(error::Kind::HeightMismatch {
                given: trusted_height,
                found: trusted_state.height(),
            });
        }

        let header_hash = self.hasher.hash_header(&trusted_state.signed_header.header);

        if header_hash != trusted_hash {
            bail!(error::Kind::HashMismatch {
                given: trusted_hash,
                found: header_hash,
            });
        }

        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }
}

impl LightClientBuilder<HasTrustedState> {
    /// TODO
    #[must_use]
    pub fn build(self) -> Instance {
        let state = State {
            light_store: self.light_store,
            verification_trace: VerificationTrace::new(),
        };

        let light_client = LightClient::from_boxed(
            self.peer_id,
            self.options,
            self.clock,
            self.scheduler,
            self.verifier,
            self.io,
        );

        Instance::new(light_client, state)
    }
}
