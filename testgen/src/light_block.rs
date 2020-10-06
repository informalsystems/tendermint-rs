use simple_error::*;
use gumdrop::Options;
use serde::{Deserialize, Serialize};

use crate::validator::generate_validators;
use crate::{Commit, Generator, Header, Validator};
use tendermint::block::signed_header::SignedHeader;
use tendermint::node::Id as PeerId;
use tendermint::validator::Set as ValidatorSet;
use tendermint::validator;
use crate::helpers::parse_as;

/// A light block is the core data structure used by the light client.
/// It records everything the light client needs to know about a block.
/// NOTE: This struct & associated `impl` below are a copy of light-client's `types.rs`.
/// The copy is necessary here to avoid a circular dependency.
/// Cf. https://github.com/informalsystems/tendermint-rs/issues/605
/// TODO: fix redundant code without introducing cyclic dependency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightBlock {
    /// Header and commit of this block
    pub signed_header: SignedHeader,
    /// Validator set at the block height
    pub validators: ValidatorSet,
    /// Validator set at the next block height
    pub next_validators: ValidatorSet,
    /// The peer ID of the node that provided this block
    pub provider: PeerId,
}

impl LightBlock {
    /// Constructs a new light block
    pub fn new(
        signed_header: SignedHeader,
        validators: ValidatorSet,
        next_validators: ValidatorSet,
        provider: PeerId,
    ) -> LightBlock {
        Self {
            signed_header,
            validators,
            next_validators,
            provider,
        }
    }
}

#[derive(Debug, Options, Deserialize, Clone)]
pub struct TestgenLightBlock {
    #[options(help = "header (required)", parse(try_from_str = "parse_as::<Header>"))]
    pub header: Option<Header>,
    #[options(help = "commit (required)", parse(try_from_str = "parse_as::<Commit>"))]
    pub commit: Option<Commit>,
    #[options(
        help = "validators (required), encoded as array of 'validator' parameters",
        parse(try_from_str = "parse_as::<Vec<Validator>>")
    )]
    pub validators: Option<Vec<Validator>>,
    #[options(
        help = "next validators (default: same as validators), encoded as array of 'validator' parameters",
        parse(try_from_str = "parse_as::<Vec<Validator>>")
    )]
    pub next_validators: Option<Vec<Validator>>,
    #[options(help = "peer id (default: peer_id())")]
    pub provider: Option<PeerId>,
}

impl TestgenLightBlock {
    /// Constructs a new Testgen-specific light block
    pub fn new(
        header: Header,
        commit: Commit,
        validators: Vec<Validator>,
        next_validators: Vec<Validator>,
        provider: PeerId,
    ) -> Self {
        Self{
            header: Some(header),
            commit: Some(commit),
            validators: Some(validators),
            next_validators: Some(next_validators),
            provider: Some(provider),
        }
    }

    pub fn new_default(validators: &[Validator], height: u64) -> Self {
        let header = Header::new(validators).height(height).chain_id("test-chain");
        let commit = Commit::new(header.clone(), 1);

        Self {
            header: Some(header),
            commit: Some(commit),
            validators: Some(validators.to_vec()),
            next_validators: None,
            provider: Some(peer_id()),
        }
    }
    set_option!(
        next_validators,
        &[Validator],
        Some(next_validators.to_vec())
    );
    set_option!(provider, PeerId);


    /// Produces a subsequent testgen light block to the supplied one
    // TODO: figure how to represent the currently ignored details in header and commit like last_block_id and other hashes
    pub fn next(&self) -> Self {
        let validators = self.validators.as_ref().expect("validator array is missing");
        let height = self
            .header.as_ref().expect("header is missing")
            .height.expect("height is missing")
            + 1;
        TestgenLightBlock::new_default(validators.as_ref(), height)
    }
}

impl std::str::FromStr for TestgenLightBlock {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let testgen_light_block = match parse_as::<TestgenLightBlock>(s) {
            Ok(input) => input,
            Err(_) => TestgenLightBlock::new_default(parse_as::<Vec<Validator>>(s)?.as_ref(), 1),
        };
        Ok(testgen_light_block)
    }
}

impl Generator<LightBlock> for TestgenLightBlock {
    fn merge_with_default(self, default: Self) -> Self {
        Self{
            header: self.header.or(default.header),
            commit: self.commit.or(default.commit),
            validators: self.validators.or(default.validators),
            next_validators: self.next_validators.or(default.next_validators),
            provider: self.provider.or(default.provider),
        }
    }

    fn generate(&self) -> Result<LightBlock, SimpleError> {
        let header = match &self.header {
            None => bail!("header is missing"),
            Some(h) => h,
        };
        let commit = match &self.commit {
            None => bail!("commit is missing"),
            Some(c) => c,
        };
        let signed_header = generate_signed_header(
            header,
            commit,
        ).expect("Could not generate signed header");

        let validators = match &self.validators {
            None => bail!("validator array is missing"),
            Some(vals) => validator::Set::new(generate_validators(vals)?),
        };

        let next_validators = match &self.next_validators {
            Some(next_vals) => validator::Set::new(generate_validators(next_vals)?),
            None => validators.clone(),
        };

        let provider = self.provider.unwrap_or(peer_id());

        let light_block = LightBlock{
            signed_header,
            validators,
            next_validators,
            provider,
        };

        Ok(light_block)
    }
}

pub fn generate_signed_header(
    raw_header: &Header,
    raw_commit: &Commit,
) -> Result<SignedHeader, SimpleError> {
    let header = match raw_header.generate() {
        Err(e) => bail!("Failed to generate header with error: {}", e),
        Ok(h) => h,
    };

    let commit = match raw_commit.generate() {
        Err(e) => bail!("Failed to generate commit with error: {}", e),
        Ok(c) => c,
    };

    Ok(SignedHeader { header, commit })
}

pub fn peer_id() -> PeerId {
    "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap()
}