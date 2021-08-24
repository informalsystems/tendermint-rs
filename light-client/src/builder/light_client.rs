//! DSL for building a light client [`Instance`]

use core::marker::PhantomData;
use tendermint::{block::Height, Hash};

use crate::builder::error::Error;
use crate::components::clock::Clock;
use crate::components::io::{AtHeight, Io};
use crate::evidence::ProdEvidenceReporter;
use crate::fork_detector::ProdForkDetector;
use crate::light_client::{LightClient, LightClientComponents, Options};
use crate::operations::Hasher;
use crate::predicates::VerificationPredicates;
use crate::state::{State, VerificationTrace};
use crate::store::LightStore;
use crate::supervisor::{Instance, SupervisorComponents};
use crate::types::{LightBlock, PeerId, Status};

#[cfg(feature = "rpc-client")]
use {
    crate::components::clock::SystemClock,
    crate::components::io::ProdIo,
    crate::components::scheduler::BasicBisectingScheduler,
    crate::components::verifier::{ProdVerifier, ProdVerifierComponents},
    crate::operations::ProdHasher,
    crate::predicates::ProdPredicates,
    std::time::Duration,
    tendermint_rpc as rpc,
};

/// No trusted state has been set yet
pub struct NoTrustedState;

/// A trusted state has been set and validated
pub struct HasTrustedState;

// pub trait LightClientBuilder {
//     type Io: Io;
//     type Clock: Clock;
//     type Hasher: Hasher;
//     type Verifier: Verifier;
//     type Scheduler: Scheduler;
//     type VerificationPredicates: VerificationPredicates;
//     type LightStore: LightStore;
// }

// pub trait LightClientBuilderWithTrustedState: LightClientBuilder {

// }

pub trait LightClientBuilderComponents: LightClientComponents {
    type VerificationPredicates: VerificationPredicates;
}

#[derive(Debug)]
pub struct ProdLightClientComponents<S: LightStore>(PhantomData<S>);

impl<S: LightStore> LightClientComponents for ProdLightClientComponents<S> {
    type Clock = SystemClock;
    type Scheduler = BasicBisectingScheduler;
    type Verifier = ProdVerifier<ProdVerifierComponents>;
    type Io = ProdIo;
    type Hasher = ProdHasher;
    type LightStore = S;
}

impl<S: LightStore> LightClientBuilderComponents for ProdLightClientComponents<S> {
    type VerificationPredicates = ProdPredicates;
}

impl<S: LightStore> SupervisorComponents for ProdLightClientComponents<S> {
    type ForkDetector = ProdForkDetector<ProdHasher>;
    type EvidenceReporter = ProdEvidenceReporter;
}

/// Builder for a light client [`Instance`]
#[must_use]
pub struct LightClientBuilder<C: LightClientBuilderComponents, State> {
    peer_id: PeerId,
    options: Options,
    io: C::Io,
    clock: C::Clock,
    hasher: C::Hasher,
    verifier: C::Verifier,
    scheduler: C::Scheduler,
    predicates: C::VerificationPredicates,
    light_store: C::LightStore,

    #[allow(dead_code)]
    state: State,
}

impl<C: LightClientBuilderComponents, Current> LightClientBuilder<C, Current> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<C, Next> {
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

impl<S: LightStore> LightClientBuilder<ProdLightClientComponents<S>, NoTrustedState> {
    /// Initialize a builder for a production (non-mock) light client.
    #[cfg(feature = "rpc-client")]
    pub fn prod(
        peer_id: PeerId,
        rpc_client: rpc::HttpClient,
        light_store: S,
        options: Options,
        timeout: Option<Duration>,
    ) -> Self {
        Self::custom(
            peer_id,
            options,
            light_store,
            ProdIo::new(peer_id, rpc_client, timeout),
            ProdHasher,
            SystemClock,
            ProdVerifier::default(),
            BasicBisectingScheduler,
            ProdPredicates,
        )
    }
}

impl<C: LightClientBuilderComponents> LightClientBuilder<C, NoTrustedState> {
    /// Initialize a builder for a custom light client, by providing all dependencies upfront.
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
        options: Options,
        light_store: C::LightStore,
        io: C::Io,
        hasher: C::Hasher,
        clock: C::Clock,
        verifier: C::Verifier,
        scheduler: C::Scheduler,
        predicates: C::VerificationPredicates,
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
    ) -> Result<LightClientBuilder<C, HasTrustedState>, Error> {
        self.validate(&trusted_state)?;

        // TODO(liamsi, romac): it is unclear if this should be Trusted or only Verified
        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// Keep using the latest verified or trusted block in the light store.
    /// Such a block must exists otherwise this will fail.
    pub fn trust_from_store(self) -> Result<LightClientBuilder<C, HasTrustedState>, Error> {
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
    ) -> Result<LightClientBuilder<C, HasTrustedState>, Error> {
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

        let header_hash = self.hasher.hash_header(&trusted_state.signed_header.header);

        if header_hash != trusted_hash {
            return Err(Error::hash_mismatch(trusted_hash, header_hash));
        }

        self.trust_light_block(trusted_state)
    }

    fn validate(&self, light_block: &LightBlock) -> Result<(), Error> {
        let header = &light_block.signed_header.header;
        let now = self.clock.now();

        self.predicates
            .is_within_trust_period(header, self.options.trusting_period, now)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .is_header_from_past(header, self.options.clock_drift, now)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .validator_sets_match(light_block, &self.hasher)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .next_validators_match(light_block, &self.hasher)
            .map_err(Error::invalid_light_block)?;

        Ok(())
    }
}

impl<C: LightClientBuilderComponents> LightClientBuilder<C, HasTrustedState> {
    /// Build the light client [`Instance`].
    #[must_use]
    pub fn build(self) -> Instance<C> {
        let state = State {
            light_store: self.light_store,
            verification_trace: VerificationTrace::new(),
        };

        let light_client = LightClient::new(
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
