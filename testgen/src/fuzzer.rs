/// A Fuzzer is anything that can produce an infinite random sequence of numbers.
/// 0 means no fuzzing, and any other number means fuzzing depending on the number.
pub trait Fuzzer {
    /// Get the next random number from the sequence
    fn next(&mut self) -> u64;

    /// Get the current (latest) number from the sequence; also refered to as the current state.
    fn current(& self) -> u64;

    /// Check if the current number is alternative 'alt' from 'total' number of alternatives.
    /// It is expected that 0 < alt <= total.
    /// If the current number is non-zero, then at least one of the alternatives will hold.
    /// If the current number is zero, none of the alternatives should hold.
    fn is_from(&self, alt: u64, total: u64) -> bool {
        self.current() % total == (alt + 1)
    }

    /// Get the random bool value encoded in the current state
    fn get_bool(&self) -> bool {
        self.current() % 2 == 1
    }

    /// Get the random u64 value encoded in the current state
    fn get_u64(&self) -> u64 {
        self.current()
    }

    /// Get the random i64 value encoded in the current state
    fn get_i64(&self) -> i64 {
        let cur = self.current();
        let max = u64::MAX / 2;
        if cur >= max {
             - ((cur % max) as i64)
        } else {
            cur as i64
        }
    }

    /// Get the random string encoded in the current state
    fn get_string(&self) -> String;
}

/// A Fuzzer that doesn't do any fuzzing (always returns 0).
pub struct NoFuzz {
}

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
    fn get_string(&self) -> String {
        String::new()
    }
}

pub struct LogFuzzer {
    fuzzer: Box<dyn Fuzzer>,
    log: Vec<u64>
}

impl LogFuzzer {
    pub fn new(fuzzer: impl Fuzzer + 'static) -> Self {
        LogFuzzer {
            fuzzer: Box::new(fuzzer),
            log: vec![]
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

    fn get_string(&self) -> String {
        self.fuzzer.get_string()
    }
}