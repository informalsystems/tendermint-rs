#[cfg(test)]
mod test {
    use crate::test::test_serialization_roundtrip;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    struct MsgUnjail {
        #[serde(alias = "address")]
        validator_addr: String,
        // NOTE: Above shouldn't be a string but: validator_addr: Vec<u8>,
        // In reality you would need to tell serde how to read bechifyed addresses instead!
        // but this is orthogonal to what this code wants to show.
    }

    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    struct MsgOtherMadeUp {
        pub test: String,
    }

    // TODO: deserves a better name
    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    struct AminoJSON<T: Serialize + DeserializeOwned> {
        #[serde(alias = "type")]
        pub type_name: String,
        #[serde(bound(deserialize = ""))]
        pub value: T,
    }

    #[test]
    fn test_simple_example() {
        let json_data =
            r#"{"type":"cosmos-sdk/MsgUnjail","value":{"address":"cosmosvaloper1v93xxeqhg9nn6"}}"#;
        let res = serde_json::from_str::<AminoJSON<MsgUnjail>>(json_data);
        println!("res: {:?}", res);
        assert!(res.is_ok());
        let msg_unjail = res.unwrap().value;
        println!("{:?}", msg_unjail);
        test_serialization_roundtrip::<AminoJSON<MsgUnjail>>(&json_data);

        let json_data2 = r#"{"type":"Foo","value":{"test":"Bar"}}"#;
        let res2 = serde_json::from_str::<AminoJSON<MsgOtherMadeUp>>(json_data2);
        println!("res: {:?}", res2);
        test_serialization_roundtrip::<AminoJSON<MsgOtherMadeUp>>(&json_data2);
    }
}
