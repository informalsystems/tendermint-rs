use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;

use crate::{helpers::*, Generator};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Time {
    #[options(help = "seconds passed since UNIX EPOCH (required; can be passed via STDIN)")]
    pub secs: Option<u64>,
}

impl Time {
    pub fn new(secs: u64) -> Self {
        Time { secs: Some(secs) }
    }
    set_option!(secs, u64);
}

impl std::str::FromStr for Time {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let time = match parse_as::<Time>(s) {
            Ok(input) => input,
            Err(_) => Time::new(try_with!(u64::from_str(s), "failed to parse time")),
        };
        Ok(time)
    }
}

impl Generator<tendermint::Time> for Time {
    fn merge_with_default(self, default: Self) -> Self {
        Time {
            secs: self.secs.or(default.secs),
        }
    }

    fn generate(&self) -> Result<tendermint::Time, SimpleError> {
        let time = match &self.secs {
            None => bail!("time is missing"),
            Some(secs) => *secs,
        };
        get_time(time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time() {
        let time = Time::new(0);
        assert_eq!(time.generate().unwrap(), tendermint::Time::unix_epoch());
    }
}
