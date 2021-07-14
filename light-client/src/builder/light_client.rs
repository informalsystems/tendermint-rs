//! DSL for building a light client [`Instance`]

use tendermint::{block::Height, Hash};

use crate::builder::error::{self, Error};
use crate::components::clock::Clock;
use crate::components::io::{AtHeight, Io};
use crate::components::scheduler::Scheduler;
use crate::components::verifier::Verifier;
use crate::light_client::{LightClient, Options};
use crate::operations::Hasher;
use crate::predicates::VerificationPredicates;
use crate::state::{State, VerificationTrace};
use crate::store::LightStore;
use crate::supervisor::Instance;
use crate::types::{LightBlock, PeerId, Status};

#[cfg(feature = "rpc-client")]
use {
    crate::components::clock::SystemClock, crate::components::io::ProdIo,
    crate::components::scheduler, crate::components::verifier::ProdVerifier,
    crate::operations::ProdHasher, crate::predicates::ProdPredicates, std::time::Duration,
    tendermint_rpc as rpc,
};

/// No trusted state has been set yet
pub struct NoTrustedState;

/// A trusted state has been set and validated
pub struct HasTrustedState;

/// Builder for a light client [`Instance`]
#[must_use]
pub struct LightClientBuilder<State> {
    peer_id: PeerId,
    options: Options,
    io: Box<dyn Io>,
    clock: Box<dyn Clock>,
    hasher: Box<dyn Hasher>,
    verifier: Box<dyn Verifier>,
    scheduler: Box<dyn Scheduler>,
    predicates: Box<dyn VerificationPredicates>,
    light_store: Box<dyn LightStore>,

    #[allow(dead_code)]
    state: State,
}

impl<Current> LightClientBuilder<Current> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<Next> {
        LightClientBuilder {
            peer_id: self.peer_id,
            options: self.options,
            io: self.io,
            clock: self.clock,
            hasher: self.hasher,
            verifier: self.verifier,
            scheduler: self.scheduler,
            predicates: self.predicates,
            light_store: self.light_store,
            state,
        }
    }
}

impl LightClientBuilder<NoTrustedState> {
    /// Initialize a builder for a production (non-mock) light client.
    #[cfg(feature = "rpc-client")]
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
            Box::new(ProdIo::new(peer_id, rpc_client, timeout)),
            Box::new(ProdHasher),
            Box::new(SystemClock),
            Box::new(ProdVerifier::default()),
            Box::new(scheduler::basic_bisecting_schedule),
            Box::new(ProdPredicates),
        )
    }

    /// Initialize a builder for a custom light client, by providing all dependencies upfront.
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
        options: Options,
        light_store: Box<dyn LightStore>,
        io: Box<dyn Io>,
        hasher: Box<dyn Hasher>,
        clock: Box<dyn Clock>,
        verifier: Box<dyn Verifier>,
        scheduler: Box<dyn Scheduler>,
        predicates: Box<dyn VerificationPredicates>,
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
            predicates,
            state: NoTrustedState,
        }
    }

    /// Set the given light block as the initial trusted state.
    fn trust_light_block(
        mut self,
        trusted_state: LightBlock,
    ) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        self.validate(&trusted_state)?;

        // TODO(liamsi, romac): it is unclear if this should be Trusted or only Verified
        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// Keep using the latest verified or trusted block in the light store.
    /// Such a block must exists otherwise this will fail.
    pub fn trust_from_store(self) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        let trusted_state = self
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(error::no_trusted_state_in_store_error)?;

        self.trust_light_block(trusted_state)
    }

    /// Set the block from the primary peer at the given height as the trusted state.
    pub fn trust_primary_at(
        self,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Result<LightClientBuilder<HasTrustedState>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(AtHeight::At(trusted_height))
            .map_err(error::io_error)?;

        if trusted_state.height() != trusted_height {
            return Err(error::height_mismatch_error(
                trusted_height,
                trusted_state.height(),
            ));
        }

        let header_hash = self.hasher.hash_header(&trusted_state.signed_header.header);

        if header_hash != trusted_hash {
            return Err(error::hash_mismatch_error(trusted_hash, header_hash));
        }

        self.trust_light_block(trusted_state)
    }

    fn validate(&self, light_block: &LightBlock) -> Result<(), Error> {
        let header = &light_block.signed_header.header;
        let now = self.clock.now();

        self.predicates
            .is_within_trust_period(header, self.options.trusting_period, now)
            .map_err(error::invalid_light_block_error)?;

        self.predicates
            .is_header_from_past(header, self.options.clock_drift, now)
            .map_err(error::invalid_light_block_error)?;

        self.predicates
            .validator_sets_match(light_block, &*self.hasher)
            .map_err(error::invalid_light_block_error)?;

        self.predicates
            .next_validators_match(light_block, &*self.hasher)
            .map_err(error::invalid_light_block_error)?;

        Ok(())
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
            self.hasher,
            self.io,
        );

        Instance::new(light_client, state)
    }
}
