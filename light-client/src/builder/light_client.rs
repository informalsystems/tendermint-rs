//! DSL for building a light client [`Instance`]

use std::time::Duration;

use tendermint::{block::Height, Hash};
use tendermint_rpc as rpc;

use crate::bail;
use crate::builder::error::{self, Error};
use crate::components::clock::{Clock, SystemClock};
use crate::components::io::{AtHeight, Io, ProdIo};
use crate::components::scheduler::{self, Scheduler};
use crate::components::verifier::{ProdVerifier, Verifier};
use crate::light_client::{LightClient, Options};
use crate::operations::{Hasher, ProdHasher};
use crate::state::{State, VerificationTrace};
use crate::store::LightStore;
use crate::supervisor::Instance;
use crate::types::{LightBlock, PeerId, Status};

/// No trusted state has been set yet
pub struct NoTrustedState;

/// A trusted state has been set and validated
pub struct HasTrustedState;

/// Builder for a light client [`Instance`]
#[must_use]
pub struct LightClientBuilder<State> {
    peer_id: PeerId,
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
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<Next> {
        LightClientBuilder {
            peer_id: self.peer_id,
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
    /// Initialize a builder for a production (non-mock) light client.
    pub fn prod(
        peer_id: PeerId,
        rpc_client: rpc::HttpClient,
        light_store: Box<dyn LightStore>,
        options: Options,
        timeout: Option<Duration>,
    ) -> Self {
        Self::custom(
            peer_id,
            options,
            light_store,
            Box::new(ProdHasher),
            Box::new(ProdIo::new(peer_id, rpc_client, timeout)),
            Box::new(ProdVerifier::default()),
            Box::new(SystemClock),
            Box::new(scheduler::basic_bisecting_schedule),
        )
    }

    /// Initialize a builder for a custom light client, by providing all dependencies upfront.
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
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

    /// Set the given light block as the initial trusted state.
    pub fn trust_light_block(
        mut self,
        trusted_state: LightBlock,
    ) -> LightClientBuilder<HasTrustedState> {
        self.light_store.insert(trusted_state, Status::Trusted);
        self.with_state(HasTrustedState)
    }

    /// Set the latest block from the primary peer as the trusted state.
    pub fn trust_primary_latest(mut self) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(AtHeight::Highest)
            .map_err(error::Kind::Io)?;

        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// Set the block from the primary peer at the given height as the trusted state.
    pub fn trust_primary_at(
        mut self,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(AtHeight::At(trusted_height))
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
    /// Build the light client [`Instance`].
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
