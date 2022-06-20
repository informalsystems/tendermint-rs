//! DSL for building a light client [`Instance`]

use sp_std::marker::PhantomData;
use tendermint::{block::Height, Hash};
use tendermint_light_client_verifier::merkle::simple_hash_from_byte_vectors;
#[cfg(feature = "rpc-client")]
use {
    crate::components::clock::SystemClock,
    crate::components::io::RpcIo,
    crate::components::scheduler,
    crate::verifier::{predicates::ProdPredicates, ProdVerifier},
    core::time::Duration,
    tendermint_rpc as rpc,
};

use crate::{
    builder::error::Error,
    components::{
        clock::Clock,
        io::{AsyncIo, AtHeight},
        scheduler::Scheduler,
    },
    light_client::LightClient,
    state::{State, VerificationTrace},
    store::LightStore,
    supervisor::Instance,
    verifier::{
        host_functions::HostFunctionsProvider,
        options::Options,
        predicates::VerificationPredicates,
        types::{LightBlock, PeerId, Status},
        Verifier,
    },
};

/// No trusted state has been set yet
pub struct NoTrustedState;

/// A trusted state has been set and validated
pub struct HasTrustedState;

/// Builder for a light client [`Instance`]
#[must_use]
pub struct LightClientBuilder<State, HostFunctions> {
    peer_id: PeerId,
    options: Options,
    io: Box<dyn AsyncIo>,
    clock: Box<dyn Clock>,
    verifier: Box<dyn Verifier>,
    scheduler: Box<dyn Scheduler>,
    predicates: Box<dyn VerificationPredicates<HostFunctions>>,
    light_store: Box<dyn LightStore>,

    #[allow(dead_code)]
    state: State,
    _phantom: PhantomData<HostFunctions>,
}

impl<Current, HostFunctions> LightClientBuilder<Current, HostFunctions> {
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<Next, HostFunctions> {
        LightClientBuilder {
            peer_id: self.peer_id,
            options: self.options,
            io: self.io,
            clock: self.clock,
            verifier: self.verifier,
            scheduler: self.scheduler,
            predicates: self.predicates,
            light_store: self.light_store,
            _phantom: PhantomData,
            state,
        }
    }
}

impl<HostFunctions> LightClientBuilder<NoTrustedState, HostFunctions>
where
    HostFunctions: HostFunctionsProvider,
{
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
            Box::new(RpcIo::new(peer_id, rpc_client, timeout)),
            Box::new(SystemClock),
            Box::new(ProdVerifier::<HostFunctions>::default()),
            Box::new(scheduler::basic_bisecting_schedule),
            Box::new(ProdPredicates::<HostFunctions>::default()),
        )
    }

    /// Initialize a builder for a custom light client, by providing all dependencies upfront.
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
        options: Options,
        light_store: Box<dyn LightStore>,
        io: Box<dyn AsyncIo>,
        clock: Box<dyn Clock>,
        verifier: Box<dyn Verifier>,
        scheduler: Box<dyn Scheduler>,
        predicates: Box<dyn VerificationPredicates<HostFunctions>>,
    ) -> Self {
        Self {
            peer_id,
            io,
            verifier,
            light_store,
            clock,
            scheduler,
            options,
            predicates,
            _phantom: PhantomData,
            state: NoTrustedState,
        }
    }

    /// Set the given light block as the initial trusted state.
    fn trust_light_block(
        mut self,
        trusted_state: LightBlock,
    ) -> Result<LightClientBuilder<HasTrustedState, HostFunctions>, Error> {
        self.validate(&trusted_state)?;

        // TODO(liamsi, romac): it is unclear if this should be Trusted or only Verified
        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// Keep using the latest verified or trusted block in the light store.
    /// Such a block must exists otherwise this will fail.
    pub fn trust_from_store(
        self,
    ) -> Result<LightClientBuilder<HasTrustedState, HostFunctions>, Error> {
        let trusted_state = self
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(Error::no_trusted_state_in_store)?;

        self.trust_light_block(trusted_state)
    }

    /// Set the block from the primary peer at the given height as the trusted state.
    pub async fn trust_primary_at(
        self,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Result<LightClientBuilder<HasTrustedState, HostFunctions>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(AtHeight::At(trusted_height))
            .await
            .map_err(Error::io)?;

        if trusted_state.height() != trusted_height {
            return Err(Error::height_mismatch(
                trusted_height,
                trusted_state.height(),
            ));
        }

        let header_hash = {
            let serialized = trusted_state.signed_header.header.serialize_to_preimage();
            Hash::Sha256(simple_hash_from_byte_vectors::<HostFunctions>(serialized))
        };

        if header_hash != trusted_hash {
            return Err(Error::hash_mismatch(trusted_hash, header_hash));
        }

        self.trust_light_block(trusted_state)
    }

    fn validate(&self, light_block: &LightBlock) -> Result<(), Error> {
        let header = &light_block.signed_header.header;
        let now = self.clock.now();

        self.predicates
            .is_within_trust_period(header.time, self.options.trusting_period, now)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .is_header_from_past(header.time, self.options.clock_drift, now)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .validator_sets_match(
                &light_block.validators,
                light_block.signed_header.header.validators_hash,
            )
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .next_validators_match(
                &light_block.next_validators,
                light_block.signed_header.header.next_validators_hash,
            )
            .map_err(Error::invalid_light_block)?;

        Ok(())
    }
}

impl<HostFunctions> LightClientBuilder<HasTrustedState, HostFunctions>
where
    HostFunctions: HostFunctionsProvider,
{
    /// Build the light client [`Instance`].
    #[must_use]
    pub fn build(self) -> Instance<HostFunctions> {
        let state = State {
            light_store: self.light_store,
            verification_trace: VerificationTrace::new(),
        };

        let light_client = LightClient::<HostFunctions>::from_boxed(
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
