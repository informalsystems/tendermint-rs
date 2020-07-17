

use serde::Serialize;
use std::str::FromStr;
use simple_error::*;

pub trait Generator<Output: Serialize>: FromStr<Err = SimpleError> {
    fn merge_with_default(&self, default: &Self) -> Self;
    fn generate(&self) -> Result<Output, SimpleError>;

    fn encode(&self) -> Result<String, SimpleError>
    {
        let res = self.generate()?;
        Ok(try_with!(
            serde_json::to_string_pretty(&res),
            "failed to serialize into JSON"
        ))
    }
}

#[macro_export]
macro_rules! gen_setter {
    ($name:ident, $t:ty) => {
        pub fn $name(&mut self, $name: $t) -> &mut Self {
        self.$name = Some($name);
        self
    }
    };
    ($name:ident, $t:ty, $val:expr) => {
        pub fn $name(&mut self, $name: $t) -> &mut Self {
        self.$name = Some($val);
        self
    }
    };
}