//! Serialization tests

use crate::test::test_serialization_roundtrip;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[test]
fn serde_integer_string() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct IntegerTests {
        #[serde(with = "super::from_str")]
        unsigned: u64,

        #[serde(with = "super::from_str")]
        signed: i64,
    }

    test_serialization_roundtrip::<IntegerTests>(
        r#"
{
  "unsigned": "9007199254740992",
  "signed": "-9007199254740994"
}
"#,
    );
}

#[test]
fn serde_duration_string() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DurationTests {
        #[serde(with = "super::time_duration")]
        duration: Duration,
    }

    test_serialization_roundtrip::<DurationTests>(
        r#"
{
  "duration": "15000000001"
}
"#,
    );
}

#[test]
fn serde_vec_string() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct BytesTests {
        #[serde(with = "super::bytes::hexstring")]
        myhexbytes: Vec<u8>,

        #[serde(with = "super::bytes::base64string")]
        mybase64bytes: Vec<u8>,

        #[serde(with = "super::bytes::string")]
        stringifiedbytes: Vec<u8>,
    }

    test_serialization_roundtrip::<BytesTests>(
        r#"
{
  "myhexbytes": "412042FF00",
  "mybase64bytes": "TXlTdHJpbmcgZGVjb2RlZC4=",
  "stringifiedbytes": "hello"
}
"#,
    );

    test_serialization_roundtrip::<BytesTests>(
        r#"
{
  "myhexbytes": null,
  "mybase64bytes": null,
  "stringifiedbytes": null
}
"#,
    );
}
