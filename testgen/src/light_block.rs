use gumdrop::Options;
use serde::{Deserialize, Serialize};
use simple_error::*;
use tendermint::{
    block::signed_header::SignedHeader, node::Id as PeerId, validator,
    validator::Set as ValidatorSet, Hash, Time,
};

use crate::{
    helpers::parse_as, validator::generate_validators, Commit, Generator, Header, Validator,
};

/// A light block is the core data structure used by the light client.
/// It records everything the light client needs to know about a block.
/// NOTE: This struct & associated `impl` below are a copy of light-client's `LightBlock`.
/// The copy is necessary here to avoid a circular dependency.
/// Cf. <https://github.com/informalsystems/tendermint-rs/issues/605>
/// TODO: fix redundant code without introducing cyclic dependency.
///
/// To convert `TmLightBlock` to the Domain type `LightBlock` used in light-client crate
/// You'll need to implement the `From` trait like below:
///
/// ```rust,ignore
/// impl From<TmLightBlock> for LightBlock {
///     fn from(tm_lb: TmLightBlock) -> Self {
///         Self {
///             signed_header: tm_lb.signed_header,
///             validators: tm_lb.validators,
///             next_validators: tm_lb.next_validators,
///             provider: tm_lb.provider,
///         }
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TmLightBlock {
    /// Header and commit of this block
    pub signed_header: SignedHeader,
    /// Validator set at the block height
    pub validators: ValidatorSet,
    /// Validator set at the next block height
    pub next_validators: ValidatorSet,
    /// The peer ID of the node that provided this block
    pub provider: PeerId,
}

/// We use this data structure as a simplistic representation of LightClient's LightBlock
#[derive(Debug, Options, Serialize, Deserialize, Clone)]
pub struct LightBlock {
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
    #[options(help = "peer id (default: default_peer_id())")]
    pub provider: Option<PeerId>,
}

impl LightBlock {
    /// Constructs a new Testgen-specific light block
    pub fn new(header: Header, commit: Commit) -> Self {
        Self {
            header: Some(header),
            commit: Some(commit),
            validators: None,
            next_validators: None,
            provider: None,
        }
    }
    set_option!(validators, &[Validator], Some(validators.to_vec()));
    set_option!(
        next_validators,
        &[Validator],
        Some(next_validators.to_vec())
    );
    set_option!(provider, &str, Some(provider.parse().unwrap()));

    pub fn new_default(height: u64) -> Self {
        let validators = [
            Validator::new("1").voting_power(50),
            Validator::new("2").voting_power(50),
        ];
        let header = Header::new(&validators)
            .height(height)
            .chain_id("test-chain")
            .next_validators(&validators)
            .time(Time::from_unix_timestamp(height as i64, 0).unwrap()); // just wanted to initialize time with some value

        let commit = Commit::new(header.clone(), 1);

        Self {
            header: Some(header),
            commit: Some(commit),
            validators: Some(validators.to_vec()),
            next_validators: Some(validators.to_vec()),
            provider: Some(default_peer_id()),
        }
    }

    pub fn new_default_with_time_and_chain_id(chain_id: String, time: Time, height: u64) -> Self {
        let validators = [
            Validator::new("1").voting_power(50),
            Validator::new("2").voting_power(50),
        ];
        let header = Header::new(&validators)
            .height(height)
            .chain_id(&chain_id)
            .next_validators(&validators)
            .time(time);

        let commit = Commit::new(header.clone(), 1);

        Self {
            header: Some(header),
            commit: Some(commit),
            validators: Some(validators.to_vec()),
            next_validators: Some(validators.to_vec()),
            provider: Some(default_peer_id()),
        }
    }

    /// Produces a subsequent, i.e. at (height+1), light block to the supplied one
    // TODO: figure how to represent the currently ignored details in header
    // TODO: and commit like last_block_id and other hashes
    pub fn next(&self) -> Self {
        let header = self.header.as_ref().expect("header is missing").next();

        let commit = Commit::new(header.clone(), 1);

        Self {
            header: Some(header),
            commit: Some(commit),
            validators: self.next_validators.clone(),
            next_validators: self.next_validators.clone(),
            provider: self.provider,
        }
    }

    /// returns the height of LightBlock's header
    pub fn height(&self) -> u64 {
        self.header
            .as_ref()
            .expect("header is missing")
            .height
            .expect("header height is missing")
    }

    /// returns the chain_id of LightBlock's header
    pub fn chain_id(&self) -> String {
        self.header
            .as_ref()
            .expect("header is missing")
            .chain_id
            .as_ref()
            .expect("chain_id is missing")
            .to_string()
    }

    /// returns the last_block_id hash of LightBlock's header
    pub fn last_block_id_hash(&self) -> Option<Hash> {
        self.header
            .as_ref()
            .expect("header is missing")
            .last_block_id_hash
    }
}

impl std::str::FromStr for LightBlock {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let light_block = match parse_as::<LightBlock>(s) {
            Ok(input) => input,
            Err(_) => LightBlock::new(parse_as::<Header>(s)?, Commit::from_str(s)?),
        };
        Ok(light_block)
    }
}

impl Generator<TmLightBlock> for LightBlock {
    fn merge_with_default(self, default: Self) -> Self {
        Self {
            header: self.header.or(default.header),
            commit: self.commit.or(default.commit),
            validators: self.validators.or(default.validators),
            next_validators: self.next_validators.or(default.next_validators),
            provider: self.provider.or(default.provider),
        }
    }

    fn generate(&self) -> Result<TmLightBlock, SimpleError> {
        let header = match &self.header {
            None => bail!("header is missing"),
            Some(h) => h,
        };
        let commit = match &self.commit {
            None => bail!("commit is missing"),
            Some(c) => c,
        };
        let signed_header =
            generate_signed_header(header, commit).expect("Could not generate signed header");

        let validators = match &self.validators {
            None => validator::Set::without_proposer(generate_validators(
                header
                    .validators
                    .as_ref()
                    .expect("missing validators in header"),
            )?),
            Some(vals) => validator::Set::without_proposer(generate_validators(vals)?),
        };

        let next_validators = match &self.next_validators {
            Some(next_vals) => validator::Set::without_proposer(generate_validators(next_vals)?),
            None => validators.clone(),
        };

        let provider = match self.provider {
            Some(peer) => peer,
            None => default_peer_id(),
        };

        let light_block = TmLightBlock {
            signed_header,
            validators,
            next_validators,
            provider,
        };

        Ok(light_block)
    }
}

/// A helper function to generate SignedHeader used by TmLightBlock
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

    Ok(SignedHeader::new(header, commit).unwrap())
}

pub fn default_peer_id() -> PeerId {
    "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_block() {
        let light_block_1 = LightBlock::new_default(1);
        let light_block_2 = LightBlock::new_default(1);

        assert_eq!(light_block_1.generate(), light_block_2.generate());

        let validators = [Validator::new("10"), Validator::new("20")];

        let light_block_3 = LightBlock::new_default(1).validators(&validators);

        assert_ne!(light_block_2.generate(), light_block_3.generate());

        let light_block_4 = LightBlock::new_default(4).validators(&validators);

        assert_ne!(light_block_3.generate(), light_block_4.generate());

        let light_block_5 = light_block_3.next();
        let lb_5_height: u64 = light_block_5
            .generate()
            .unwrap()
            .signed_header
            .header
            .height
            .into();

        assert_eq!(2, lb_5_height);

        let header_6 = Header::new(&validators)
            .next_validators(&validators)
            .height(10)
            .time(Time::from_unix_timestamp(10, 0).unwrap())
            .chain_id("test-chain");
        let commit_6 = Commit::new(header_6.clone(), 1);
        let light_block_6 = LightBlock::new(header_6.clone(), commit_6);

        let header_7 = header_6.next();
        let commit_7 = Commit::new(header_7.clone(), 1);
        let light_block_7 = LightBlock::new(header_7, commit_7);

        assert_eq!(light_block_7.height(), 11);
        assert_eq!(light_block_7.chain_id(), "test-chain");
        assert_ne!(light_block_6.generate(), light_block_7.generate());
    }
}
