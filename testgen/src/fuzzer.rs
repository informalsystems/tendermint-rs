use serde::{Serialize};
use rand_pcg::Pcg64 as Pcg;
use rand::{SeedableRng, Rng, RngCore};
use rand::distributions::{Alphanumeric};

/// A Fuzzer is anything that can produce an infinite random sequence of numbers.
/// 0 means no fuzzing, and any other number means fuzzing depending on the number.
pub trait Fuzzer {
    /// Get the next random number from the sequence
    fn next(&mut self) -> u64;

    /// Get the current (latest) number from the sequence; also referred to as the current state.
    /// This is valid only after calling next() at least once!
    fn current(&self) -> u64;

    /// Check if the current number is alternative 'alt' from 'total' number of alternatives.
    /// It is expected that 0 < alt <= total.
    /// If the current number is non-zero, then at least one of the alternatives will hold.
    /// If the current number is zero, none of the alternatives should hold.
    fn is_from(&self, alt: u64, total: u64) -> bool {
        let _rng = self.current_rng_default();
        if self.current() == 0 {
            false
        } else {
            (self.current() - 1) % total == alt - 1
        }
    }

    /// Get the random number generator seeded from the current state and the index
    fn current_rng(&self, index: u64) -> Pcg {
        Pcg::new(self.current() as u128, index as u128)
    }

    /// Get the random number generator seeded from the current state and the index
    fn current_rng_default(&self) -> Pcg {
        self.current_rng(0xcafef00dd15ea5e5)
    }


    /// Get indexed random bool value from the current state
    fn get_bool(&self, index: u64) -> bool { self.current_rng(index).gen() }

    /// Get indexed random u64 value from the current state
    fn get_u64(&self, index: u64) -> u64 { self.current_rng(index).gen() }

    /// Get indexed random i64 value from the current state
    fn get_i64(&self, index: u64) -> i64 { self.current_rng(index).gen() }

    /// Get the indexed random string from the current state
    fn get_string(&self, index: u64, length: usize) -> String {
        self.current_rng(index).sample_iter(Alphanumeric).take(length).collect()
    }
}

pub fn fuzz_vector<T>(fuzzer: &impl Fuzzer, vec: &mut Vec<T>, val: T) {
    if fuzzer.get_bool(0) {
        vec.push(val)
    } else if !vec.is_empty() {
        let i = (fuzzer.get_u64(0) as usize) % vec.len();
        vec.remove(i);
    }
}

/// A Fuzzer that doesn't do any fuzzing (always returns 0).
pub struct NoFuzz {}

impl NoFuzz {
    pub fn new() -> Self {
        NoFuzz {}
    }
}

impl Fuzzer for NoFuzz {
    fn next(&mut self) -> u64 {
        0
    }
    fn current(&self) -> u64 {
        0
    }
}

impl Default for NoFuzz {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogFuzzer {
    fuzzer: Box<dyn Fuzzer>,
    log: Vec<u64>,
}

impl LogFuzzer {
    pub fn new(fuzzer: impl Fuzzer + 'static) -> Self {
        LogFuzzer {
            fuzzer: Box::new(fuzzer),
            log: vec![],
        }
    }
}

impl Fuzzer for LogFuzzer {
    fn next(&mut self) -> u64 {
        let next = self.fuzzer.next();
        self.log.push(next);
        next
    }

    fn current(&self) -> u64 {
        self.fuzzer.current()
    }
}

pub struct RepeatFuzzer {
    repeat: Vec<u64>,
    current: usize,
}

impl RepeatFuzzer {
    pub fn new(repeat: &[u64]) -> Self {
        RepeatFuzzer {
            repeat: repeat.to_vec(),
            current: 0,
        }
    }
}

impl Fuzzer for RepeatFuzzer {
    fn next(&mut self) -> u64 {
        if self.current < self.repeat.len() - 1 {
            self.current += 1;
        } else {
            self.current = 0;
        }
        self.current()
    }

    fn current(&self) -> u64 {
        self.repeat[self.current]
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RandomFuzzer {
    seed: u64,
    step: u64,
    #[serde(skip)]
    current: u64,
    #[serde(skip)]
    rng: Pcg,
}

impl RandomFuzzer {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            step: 0,
            current: 0,
            rng: Pcg::seed_from_u64(seed),
        }
    }

    pub fn reset(&mut self) {
        self.rng = Pcg::seed_from_u64(self.seed);
        self.step = 0;
        self.current = 0;
    }

    pub fn step(&self) -> u64 {
        self.step
    }

    pub fn goto(&mut self, step: u64) {
        let steps = if step >= self.step {
            step - self.step
        } else {
            self.reset();
            step
        };
        for _i in 0..steps {
            self.next();
        }
    }
}

impl Fuzzer for RandomFuzzer {
    fn next(&mut self) -> u64 {
        self.current = self.rng.next_u64();
        self.step += 1;
        self.current
    }

    fn current(&self) -> u64 {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_fuzzer() {
        let mut fuzzer = RepeatFuzzer::new(&[0, 1, 2]);
        assert_eq!(fuzzer.next(), 1);
        assert_eq!(fuzzer.current(), 1);
        assert_eq!(fuzzer.next(), 2);
        assert_eq!(fuzzer.current(), 2);
        assert_eq!(fuzzer.next(), 0);
        assert_eq!(fuzzer.current(), 0);
    }

    #[test]
    fn test_random_fuzzer() {
        let mut fuzzer = RandomFuzzer::new(0);
        let s1 = fuzzer.next();
        assert_eq!(fuzzer.current(), s1);
        let s2 = fuzzer.next();
        assert_eq!(fuzzer.current(), s2);
        let s3 = fuzzer.next();
        assert_eq!(fuzzer.current(), s3);
        assert_eq!(fuzzer.step(), 3);

        let s30u64 = fuzzer.get_u64(0);
        let s31u64 = fuzzer.get_u64(1);
        assert_ne!(s30u64, s31u64);

        let s30string = fuzzer.get_string(0, 10);
        assert_eq!(s30string.len(), 10);
        let s31string = fuzzer.get_string(1, 20);
        assert_eq!(s31string.len(), 20);

        // test reproducibility of results
        fuzzer.next();
        fuzzer.goto(3);
        assert_eq!(fuzzer.get_u64(0), s30u64);
        assert_eq!(fuzzer.get_u64(1), s31u64);
        assert_eq!(fuzzer.get_string(0, 10), s30string);
        assert_eq!(fuzzer.get_string(1, 20), s31string);
        let mut fuzzer = RandomFuzzer::new(0);
        fuzzer.goto(3);
        assert_eq!(fuzzer.get_u64(0), s30u64);
        assert_eq!(fuzzer.get_u64(1), s31u64);
        assert_eq!(fuzzer.get_string(0, 10), s30string);
        assert_eq!(fuzzer.get_string(1, 20), s31string);
    }

}
