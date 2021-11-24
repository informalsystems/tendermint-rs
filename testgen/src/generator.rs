use serde::Serialize;
use simple_error::*;
use std::str::FromStr;

/// A trait that allows to generate complex objects from simple companion objects.
/// A companion type should have a simple API, leaving most fields optional.
pub trait Generator<Output: Serialize>: FromStr<Err = SimpleError> + Clone {
    /// Merge this companion with the another, default one.
    /// The options present in this object will override those in the default one.
    fn merge_with_default(self, default: Self) -> Self;

    /// Generate the complex object from this companion object.
    fn generate(&self) -> Result<Output, SimpleError>;

    /// Generate and serialize the complex object
    fn encode(&self) -> Result<String, SimpleError> {
        let res = self.generate()?;
        serde_json::to_string_pretty(&res)
            .map_err(|_| SimpleError::new("failed to serialize into JSON".to_string()))
    }
}
