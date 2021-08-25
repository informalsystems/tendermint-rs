//! DSL for building a light client [`Instance`]

use tendermint::{block::Height, Hash};

use crate::builder::error::Error;
use crate::components::clock::Clock;
use crate::components::io::{AtHeight, Io};
use crate::components::scheduler::Scheduler;
use crate::components::verifier::Verifier;
use crate::light_client::{LightClientImpl, Options};
use crate::predicates;
use crate::state::{State, VerificationTrace};
use crate::store::LightStore;
use crate::supervisor::Instance;
use crate::types::{LightBlock, PeerId, Status};

#[cfg(feature = "rpc-client")]
use {
    crate::components::clock::SystemClock, crate::components::io::ProdIo,
    crate::components::scheduler::BasicBisectingScheduler,
    crate::components::verifier::ProdVerifier, std::time::Duration, tendermint_rpc as rpc,
};

/// No trusted state has been set yet
pub struct NoTrustedState;

/// A trusted state has been set and validated
pub struct HasTrustedState;

/// Builder for a light client [`Instance`]
#[must_use]
pub struct LightClientBuilder<State, C, S, V, I, L> {
    peer_id: PeerId,
    options: Options,

    clock: C,
    scheduler: S,
    verifier: V,
    io: I,
    light_store: L,

    #[allow(dead_code)]
    state: State,
}

impl<Current, C, S, V, I, L> LightClientBuilder<Current, C, S, V, I, L> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<Next, C, S, V, I, L> {
        LightClientBuilder {
            peer_id: self.peer_id,
            options: self.options,
            io: self.io,
            clock: self.clock,
            verifier: self.verifier,
            scheduler: self.scheduler,
            light_store: self.light_store,
            state,
        }
    }
}

#[cfg(feature = "rpc-client")]
impl<L: LightStore>
    LightClientBuilder<
        NoTrustedState,
        SystemClock,
        BasicBisectingScheduler,
        ProdVerifier,
        ProdIo,
        L,
    >
{
    /// Initialize a builder for a production (non-mock) light client.
    pub fn prod(
        peer_id: PeerId,
        rpc_client: rpc::HttpClient,
        light_store: L,
        options: Options,
        timeout: Option<Duration>,
    ) -> Self {
        Self::custom(
            peer_id,
            options,
            light_store,
            ProdIo::new(peer_id, rpc_client, timeout),
            SystemClock,
            ProdVerifier::default(),
            BasicBisectingScheduler,
        )
    }
}

impl<C, S, V, I, L> LightClientBuilder<NoTrustedState, C, S, V, I, L>
where
    C: Clock,
    V: Verifier,
    S: Scheduler,
    I: Io,
    L: LightStore,
{
    /// Initialize a builder for a custom light client, by providing all dependencies upfront.
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
        options: Options,
        light_store: L,
        io: I,
        clock: C,
        verifier: V,
        scheduler: S,
    ) -> Self {
        Self {
            peer_id,
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
    fn trust_light_block(
        mut self,
        trusted_state: LightBlock,
    ) -> Result<LightClientBuilder<HasTrustedState, C, S, V, I, L>, Error> {
        self.validate(&trusted_state)?;

        // TODO(liamsi, romac): it is unclear if this should be Trusted or only Verified
        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// Keep using the latest verified or trusted block in the light store.
    /// Such a block must exists otherwise this will fail.
    pub fn trust_from_store(
        self,
    ) -> Result<LightClientBuilder<HasTrustedState, C, S, V, I, L>, Error> {
        let trusted_state = self
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(Error::no_trusted_state_in_store)?;

        self.trust_light_block(trusted_state)
    }

    /// Set the block from the primary peer at the given height as the trusted state.
    pub fn trust_primary_at(
        self,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Result<LightClientBuilder<HasTrustedState, C, S, V, I, L>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(AtHeight::At(trusted_height))
            .map_err(Error::io)?;

        if trusted_state.height() != trusted_height {
            return Err(Error::height_mismatch(
                trusted_height,
                trusted_state.height(),
            ));
        }

        let header_hash = trusted_state.signed_header.header.hash();

        if header_hash != trusted_hash {
            return Err(Error::hash_mismatch(trusted_hash, header_hash));
        }

        self.trust_light_block(trusted_state)
    }

    fn validate(&self, light_block: &LightBlock) -> Result<(), Error> {
        let header = &light_block.signed_header.header;
        let now = self.clock.now();

        predicates::is_within_trust_period(header, self.options.trusting_period, now)
            .map_err(Error::invalid_light_block)?;

        predicates::is_header_from_past(header, self.options.clock_drift, now)
            .map_err(Error::invalid_light_block)?;

        predicates::validator_sets_match(light_block).map_err(Error::invalid_light_block)?;

        predicates::next_validators_match(light_block).map_err(Error::invalid_light_block)?;

        Ok(())
    }
}

impl<C, S, V, I, L> LightClientBuilder<HasTrustedState, C, S, V, I, L>
where
    C: Clock,
    V: Verifier,
    S: Scheduler,
    I: Io,
    L: LightStore,
{
    /// Build the light client [`Instance`].
    #[must_use]
    pub fn build(self) -> Instance<LightClientImpl<C, S, V, I>, L> {
        let state = State {
            light_store: self.light_store,
            verification_trace: VerificationTrace::new(),
        };

        let light_client = LightClientImpl::new(
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
