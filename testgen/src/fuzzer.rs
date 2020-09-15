/// A Fuzzer is anything that can produce an infinite random sequence of numbers.
/// 0 means no fuzzing, and any other number means fuzzing depending on the number.
pub trait Fuzzer {
    fn next(&mut self) -> u64;
}

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
}