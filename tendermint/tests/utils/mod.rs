use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::{fs, path::PathBuf};

pub mod apalache;
pub mod command;
pub mod jsonatr;
pub mod lite;

/// Test that a struct `T` can be:
///
/// - serialized to JSON
/// - parsed back from the serialized JSON of the previous step
/// - that the two parsed structs are equal according to their `PartialEq` impl
pub fn test_serialization_roundtrip<T>(obj: &T)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_string(obj).unwrap();
    let parsed = serde_json::from_str(&serialized).unwrap();
    assert_eq!(obj, &parsed);
}

/// Read a file into a string
pub fn read_file(dir: &str, file: &str) -> String {
    fs::read_to_string(PathBuf::from(dir.to_owned()).join(file)).unwrap()
}

/// Tries to parse a string as the given type
pub fn parse_as<T: DeserializeOwned>(input: &str) -> Option<T> {
    match serde_json::from_str(input) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}
