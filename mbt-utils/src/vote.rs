use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use crate::helpers::*;
use tendermint::{Time};
use crate::{Validator, Header};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Vote {
    #[options(help = "validator of this vote (required; can be passed via STDIN)",
      parse(try_from_str = "parse_as::<Validator>"))]
    pub validator: Option<Validator>,
    #[options(help = "validator index (default: from commit header)")]
    pub index: Option<u64>,
    #[options(help = "header to sign (default: commit header)")]
    pub header: Option<Header>,
    #[options(help = "vote type; 'precommit' if set, otherwise 'prevote' (default)")]
    pub precommit: Option<()>,
    #[options(help = "block height (default: from commit header)")]
    pub height: Option<u64>,
    #[options(help = "time (default: from commit header)")]
    pub time: Option<Time>,
    #[options(help = "commit round (default: from commit)")]
    pub round: Option<u64>,
}

impl Vote {
    pub fn new(validator: &Validator) -> Self {
        Vote {
            validator: Some(validator.clone()),
            index: None,
            header: None,
            precommit: None,
            height: None,
            time: None,
            round: None
        }
    }
    set_option!(index, u64);
    set_option!(precommit, ());
    set_option!(height, u64);
    set_option!(time, Time);
    set_option!(round, u64);
}

impl std::str::FromStr for Vote {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vote = match parse_as::<Vote>(s) {
            Ok(input) => input,
            Err(_) => Vote::new(&parse_as::<Validator>(s)?)
        };
        Ok(vote)
    }
}
