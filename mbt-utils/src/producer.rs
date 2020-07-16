use serde::Serialize;
use std::str::FromStr;
use simple_error::*;

pub trait Producer<Output: Serialize>: FromStr<Err = SimpleError> {
    fn merge_with_default(&self, default: &Self) -> Self;
    fn produce(&self) -> Result<Output, SimpleError>;

    fn encode(&self) -> Result<String, SimpleError>
    {
        let res = self.produce()?;
        Ok(try_with!(
            serde_json::to_string_pretty(&res),
            "failed to serialize into JSON"
        ))
    }
}
