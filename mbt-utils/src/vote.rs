use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use crate::helpers::*;
use crate::validator::Validator;

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Vote {
    #[options(help = "validator of this vote (required; can be passed via STDIN)",
      parse(try_from_str = "parse_as::<Validator>"))]
    pub validator: Option<Validator>,
    #[options(help = "vote type; if set -- this is 'precommit' vote, otherwise 'prevote' (default)")]
    pub precommit: Option<bool>,
    #[options(help = "block height (default: taken from block)")]
    pub height: Option<u64>,
}

impl Vote {
    pub fn new(validator: &Validator) -> Self {
        Vote {
            validator: Some(validator.clone()),
            precommit: None,
            height: None
        }
    }
    set_option!(precommit, bool);
    set_option!(height, u64);
}

impl std::str::FromStr for Vote {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vote = match parse_as::<Vote>(s) {
            Ok(input) => input,
            Err(_) => Vote {
                validator: Some(parse_as::<Validator>(s)?),
                precommit: None,
                height: None
            }
        };
        Ok(vote)
    }
}
