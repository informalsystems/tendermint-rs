//! Serialization tests

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct IntegerTests {
    #[serde(with = "super::from_str")]
    unsigned: u64,

    #[serde(with = "super::from_str")]
    signed: i64,
}

#[derive(Serialize, Deserialize)]
struct DurationTests {
    #[serde(with = "super::time_duration")]
    duration: Duration,
}

#[derive(Serialize, Deserialize)]
struct BytesTests {
    #[serde(with = "super::bytes::hexstring")]
    myhexbytes: Vec<u8>,

    #[serde(with = "super::bytes::base64string")]
    mybase64bytes: Vec<u8>,

    #[serde(with = "super::bytes::string")]
    stringifiedbytes: Vec<u8>,
}

#[test]
fn serialize_integer_into_string() {
    let outgoing = IntegerTests {
        unsigned: 9_007_199_254_740_995,
        signed: -9_007_199_254_740_997,
    };

    let result: String = serde_json::to_string(&outgoing).unwrap();

    assert_eq!(
        result,
        r#"{"unsigned":"9007199254740995","signed":"-9007199254740997"}"#
    );
}

#[test]
fn deserialize_integer_from_string() {
    let incoming = r#"
{
  "unsigned": "9007199254740992",
  "signed": "-9007199254740994"
}
"#;

    let result: IntegerTests = serde_json::from_str(&incoming).unwrap();

    assert_eq!(result.unsigned, 9_007_199_254_740_992);
    assert_eq!(result.signed, -9_007_199_254_740_994);
}

#[test]
fn serialize_duration_into_string() {
    let outgoing = DurationTests {
        duration: Duration::from_secs(5),
    };

    let result: String = serde_json::to_string(&outgoing).unwrap();

    assert_eq!(result, r#"{"duration":"5000000000"}"#);
}

#[test]
fn deserialize_duration_from_string() {
    let incoming = r#"
{
  "duration": "15000000001"
}
"#;

    let result: DurationTests = serde_json::from_str(&incoming).unwrap();

    assert_eq!(result.duration.as_secs(), 15);
    assert_eq!(result.duration.as_nanos(), 15_000_000_001);
}

#[test]
fn serialize_vec_into_string() {
    let outgoing = BytesTests {
        myhexbytes: vec![00, 255, 32],
        mybase64bytes: b"MyString encoded.".to_vec(),
        stringifiedbytes: vec![65, 66, 67],
    };

    let result: String = serde_json::to_string(&outgoing).unwrap();

    assert_eq!(
        result,
        r#"{"myhexbytes":"00ff20","mybase64bytes":"TXlTdHJpbmcgZW5jb2RlZC4=","stringifiedbytes":"ABC"}"#
    );
}

#[test]
fn deserialize_vec_from_string() {
    let incoming = r#"
{
  "myhexbytes": "412042FF00",
  "mybase64bytes": "TXlTdHJpbmcgZGVjb2RlZC4=",
  "stringifiedbytes": "hello"
}
"#;

    let result: BytesTests = serde_json::from_str(&incoming).unwrap();

    assert_eq!(result.myhexbytes, vec![65, 32, 66, 255, 0]);
    assert_eq!(result.mybase64bytes, b"MyString decoded.");
    assert_eq!(result.stringifiedbytes, b"hello");
}

#[test]
fn serialize_emptyvec_into_emptystring() {
    let outgoing = BytesTests {
        myhexbytes: vec![],
        mybase64bytes: vec![],
        stringifiedbytes: vec![],
    };

    let result: String = serde_json::to_string(&outgoing).unwrap();

    assert_eq!(
        result,
        r#"{"myhexbytes":"","mybase64bytes":"","stringifiedbytes":""}"#
    );
}

#[test]
fn deserialize_emptyvec_from_null() {
    let incoming = r#"
{
  "myhexbytes": null,
  "mybase64bytes": null,
  "stringifiedbytes": null
}
"#;

    let result: BytesTests = serde_json::from_str(&incoming).unwrap();

    assert_eq!(result.myhexbytes, Vec::<u8>::new());
    assert_eq!(result.mybase64bytes, Vec::<u8>::new());
    assert_eq!(result.stringifiedbytes, Vec::<u8>::new());
}
