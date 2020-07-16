use serde::Serialize;
use std::str::FromStr;
use simple_error::*;

pub trait Producer<Output: Serialize>: FromStr {
    fn parse_stdin() -> Result<Self, SimpleError>
    where
        Self: std::marker::Sized;

    fn merge_with_default(&self, other: &Self) -> Self;

    fn produce(&self) -> Result<Output, SimpleError>;

    fn encode(&self) -> Result<String, SimpleError>
    where
        Self: std::marker::Sized,
    {
        let res = self.produce()?;
        Ok(try_with!(
            serde_json::to_string_pretty(&res),
            "failed to serialize into JSON"
        ))
    }

    fn encode_with_stdin(&self) -> Result<String, SimpleError>
    where
        Self: std::marker::Sized,
    {
        let stdin = Self::parse_stdin()?;
        let producer = self.merge_with_default(&stdin);
        producer.encode()
    }
}
