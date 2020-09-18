use crate::{fuzzer, helpers::*, Generator};
use gumdrop::Options;
use serde::{Deserialize, Serialize};
use simple_error::*;
use tendermint::Time as TMTime;

#[derive(Debug, Options, Serialize, Deserialize, Clone)]
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

impl Generator<TMTime> for Time {
    fn merge_with_default(self, default: Self) -> Self {
        Time {
            secs: self.secs.or(default.secs),
        }
    }

    fn fuzz(&self, _fuzzer: &mut impl fuzzer::Fuzzer) -> Self {
        self.clone()
    }

    fn generate_fuzz(&self, fuzzer: &mut impl fuzzer::Fuzzer) -> Result<TMTime, SimpleError> {
        fuzzer.next();
        let time = if fuzzer.is_from(1, 1) {
            let x = fuzzer.get_u64(0);
            println!("x = {}", x);
            x

        } else {
            match &self.secs {
                None => bail!("time is missing"),
                Some(secs) => *secs,
            }
        };
        Ok(get_time(time))
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

    #[test]
    fn test_time_fuzz() {
        let mut fuzzer = fuzzer::RepeatFuzzer::new(&[1, 0]);
        let time = Time::new(0);
        assert_eq!(
            time.generate_fuzz(&mut fuzzer).unwrap(),
            tendermint::Time::unix_epoch()
        );
        // TODO This test fails: https://github.com/informalsystems/tendermint-rs/issues/575
        // assert_ne!(
        //     time.generate_fuzz(&mut fuzzer).unwrap(),
        //     tendermint::Time::unix_epoch()
        // );
    }
}
