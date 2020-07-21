//! Helper functions

use serde::de::DeserializeOwned;
use simple_error::*;
use std::io::{self, Read};

#[macro_export]
macro_rules! set_option {
    ($name:ident, $t:ty) => {
        pub fn $name(mut self, $name: $t) -> Self {
        self.$name = Some($name.clone());
        self
    }
    };
    ($name:ident, $t:ty, $val:expr) => {
        pub fn $name(mut self, $name: $t) -> Self {
        self.$name = $val;
        self
    }
    };
}

/// Tries to parse a string as the given type; otherwise returns the input wrapped in SimpleError
pub fn parse_as<T: DeserializeOwned>(input: &str) -> Result<T, SimpleError> {
    match serde_json::from_str(input) {
        Ok(res) => Ok(res),
        Err(_) => Err(SimpleError::new(input)),
    }
}

pub fn read_stdin() -> Result<String, SimpleError> {
    let mut buffer = String::new();
    try_with!(io::stdin().read_to_string(&mut buffer), "");
    Ok(buffer)
}


/// Tries to parse STDIN as the given type; otherwise returns the input wrapped in SimpleError
pub fn parse_stdin_as<T: DeserializeOwned>() -> Result<T, SimpleError> {
    let mut buffer = String::new();
    match io::stdin().read_to_string(&mut buffer) {
        Err(_) => Err(SimpleError::new("")),
        Ok(_) => parse_as::<T>(&buffer),
    }
}

pub fn choose_or<T>(input: Option<T>, default: T) -> T {
    if let Some(x) = input {
        x
    } else {
        default
    }
}

pub fn choose_from<T: Clone>(cli: &Option<T>, input: &Option<T>) -> Option<T> {
    if let Some(x) = cli {
        Some(x.clone())
    } else if let Some(y) = input {
        Some(y.clone())
    } else {
        None
    }
}
