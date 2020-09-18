use crate::{fuzzer, Fuzzer};
use serde::Serialize;
use simple_error::*;
use std::str::FromStr;

/// A trait that allows to generate complex objects from simple companion objects.
/// A companion type should have a simple API, leaving most fields optional.
pub trait Generator<Output: Serialize>: FromStr<Err = SimpleError> + Clone + Serialize {
    /// Merge this companion with the another, default one.
    /// The options present in this object will override those in the default one.
    fn merge_with_default(self, default: Self) -> Self;

    /// Generate the complex object from this companion object.
    fn generate(&self) -> Result<Output, SimpleError> {
        self.generate_fuzz(&mut fuzzer::NoFuzz::new())
    }

    /// Generate the complex object from this companion object, and encode it into JSON.
    fn encode(&self) -> Result<String, SimpleError> {
        let res = self.generate()?;
        Ok(try_with!(
            serde_json::to_string_pretty(&res),
            "failed to serialize into JSON"
        ))
    }

    /// Encode this companion object into JSON.
    fn encode_input(&self) -> Result<String, SimpleError> {
        Ok(try_with!(
            serde_json::to_string_pretty(self),
            "failed to serialize into JSON"
        ))
    }

    /// Fuzz this companion object.
    fn fuzz(&self, fuzzer: &mut dyn Fuzzer) -> Self;

    /// Generate the complex object from this companion object, and fuzz its representation.
    fn generate_fuzz(&self, fuzzer: &mut dyn Fuzzer) -> Result<Output, SimpleError>;
}
