/// A Fuzzer is anything that can produce an infinite random sequence of numbers.
/// 0 means no fuzzing, and any other number means fuzzing depending on the number.
pub trait Fuzzer {
    /// Get the next random number from the sequence
    fn next(&mut self) -> u64;

    /// Get the current (latest) number from the sequence; also refered to as the current state.
    fn current(&self) -> u64;

    /// Check if the current number is alternative 'alt' from 'total' number of alternatives.
    /// It is expected that 0 < alt <= total.
    /// If the current number is non-zero, then at least one of the alternatives will hold.
    /// If the current number is zero, none of the alternatives should hold.
    fn is_from(&self, alt: u64, total: u64) -> bool {
        if self.current() == 0 {
            false
        } else {
            (self.current() - 1) % total == alt - 1
        }
    }

    /// Get indexed random bool value from the current state
    fn get_bool(&self, index: u64) -> bool {
        self.current() + index % 2 == 1
    }

    /// Get indexed random i64 value from the current state
    fn get_u64(&self, index: u64) -> u64 {
        self.current() + index
    }

    /// Get indexed random i64 value from the current state
    fn get_i64(&self, index: u64) -> i64 {
        let cur = self.current() + index;
        let max = u64::MAX / 2;
        if cur >= max {
            -((cur % max) as i64)
        } else {
            cur as i64
        }
    }

    /// Get the indexed random string from the current state
    fn get_string(&self, index: u64) -> String;
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
    fn get_string(&self, index: u64) -> String {
        index.to_string()
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

    fn get_string(&self, index: u64) -> String {
        self.fuzzer.get_string(index)
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

    fn get_string(&self, index: u64) -> String {
        (self.current() + index).to_string()
    }
}
