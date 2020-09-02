use std::convert::TryFrom;
use tendermint_proto::types::BlockId;
use tendermint_proto::types::PartSetHeader;
use tendermint_proto::DomainType;

// Example implementation of a Domain Type.
#[derive(DomainType, Clone)]
#[rawtype(BlockId)]
pub struct DomainBlockId {
    hash: String,
    part_set_header_exists: bool,
}

// DomainTypes MUST have the TryFrom trait to convert from RawTypes.
impl TryFrom<BlockId> for DomainBlockId {
    type Error = &'static str;

    fn try_from(value: BlockId) -> Result<Self, Self::Error> {
        Ok(DomainBlockId {
            hash: String::from_utf8(value.hash)
                .map_err(|_| "Could not convert vector to string")?,
            part_set_header_exists: value.part_set_header != None,
        })
    }
}

// DomainTypes MUST be able to convert to RawTypes without errors using the From trait.
impl From<DomainBlockId> for BlockId {
    fn from(value: DomainBlockId) -> Self {
        BlockId {
            hash: value.hash.into_bytes(),
            part_set_header: match value.part_set_header_exists {
                true => Some(PartSetHeader {
                    total: 0,
                    hash: vec![],
                }),
                false => None,
            },
        }
    }
}

#[test]
pub fn codec_example() {
    let my_domain_type = DomainBlockId {
        hash: "Hello world!".to_string(),
        part_set_header_exists: false,
    };

    let mut wire = vec![];
    my_domain_type.clone().encode(&mut wire).unwrap();
    assert_eq!(
        wire,
        vec![10, 12, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33]
    );
    let new_domain_type = DomainBlockId::decode(wire.as_ref()).unwrap();
    assert_eq!(new_domain_type.hash, "Hello world!".to_string());
    assert_eq!(new_domain_type.part_set_header_exists, false);
    assert_eq!(my_domain_type.clone().encoded_len(), 14);
}

#[test]
pub fn codec_length_delimited_example() {
    let my_domain_type = DomainBlockId {
        hash: "Hello world!".to_string(),
        part_set_header_exists: false,
    };

    let mut wire = vec![];
    my_domain_type.encode_length_delimited(&mut wire).unwrap();
    assert_eq!(
        wire,
        vec![14, 10, 12, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33]
    );

    let new_domain_type = DomainBlockId::decode_length_delimited(wire.as_ref()).unwrap();
    assert_eq!(new_domain_type.hash, "Hello world!".to_string());
    assert_eq!(new_domain_type.part_set_header_exists, false);
}
