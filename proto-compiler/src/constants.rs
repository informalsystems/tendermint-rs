/// Tendermint protobuf version
pub const TENDERMINT_REPO: &str = "https://github.com/tendermint/tendermint";
pub const TENDERMINT_COMMITISH: &str = "tags/v0.34.0-rc5";

/// Predefined custom attributes for field annotations
const FROM_STR: &str = r#"#[serde(with = "crate::serializers::from_str")]"#;
const FROM_STR_DEFAULT: &str = r#"#[serde(with = "crate::serializers::from_str", default)]"#;
const HEXSTRING: &str = r#"#[serde(with = "crate::serializers::bytes::hexstring")]"#;
const VEC_SKIP_IF_EMPTY: &str =
    r#"#[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]"#;
const RENAME_PARTS: &str = r#"#[serde(rename = "parts")]"#;

/// Predefined custom attributes for message annotations
const PRIMITIVE_ENUM: &str = r#"#[derive(::num_derive::FromPrimitive, ::num_derive::ToPrimitive)]"#;
const SERIALIZE: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;

/// Custom type attributes applied on top of protobuf structs
/// The first item in the tuple defines the message where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".tendermint.types.BlockIDFlag", PRIMITIVE_ENUM),
    (".tendermint.types.Block", SERIALIZE),
    (".tendermint.types.Data", SERIALIZE),
    (".tendermint.types.EvidenceData", SERIALIZE),
    (".tendermint.types.Evidence", SERIALIZE),
    (".tendermint.types.evidence.Sum", SERIALIZE),
    (".tendermint.types.DuplicateVoteEvidence", SERIALIZE),
    (".tendermint.types.Vote", SERIALIZE),
    (".tendermint.types.BlockID", SERIALIZE),
    (".tendermint.types.PartSetHeader", SERIALIZE),
    (".google.protobuf.Timestamp", SERIALIZE),
    (".tendermint.types.LightClientAttackEvidence", SERIALIZE),
    (".tendermint.types.LightBlock", SERIALIZE),
    (".tendermint.types.SignedHeader", SERIALIZE),
    (".tendermint.types.Header", SERIALIZE),
    (".tendermint.version.Consensus", SERIALIZE),
    (".tendermint.types.Commit", SERIALIZE),
    (".tendermint.types.CommitSig", SERIALIZE),
    (".tendermint.types.ValidatorSet", SERIALIZE),
    (".tendermint.crypto.PublicKey", SERIALIZE),

    (".tendermint.abci.ResponseInfo", SERIALIZE),
    (".tendermint.types.CanonicalBlockID", SERIALIZE),
    (".tendermint.types.CanonicalPartSetHeader", SERIALIZE),
    (".tendermint.types.Validator", SERIALIZE),
    (".tendermint.types.CanonicalVote", SERIALIZE),
];

/// Custom field attributes applied on top of protobuf fields in (a) struct(s)
/// The first item in the tuple defines the field where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    (".tendermint.abci.ResponseInfo.last_block_height", FROM_STR),
    (".tendermint.version.Consensus.block", FROM_STR),
    (".tendermint.version.Consensus.app", FROM_STR_DEFAULT),
    (
        ".tendermint.abci.ResponseInfo.last_block_app_hash",
        VEC_SKIP_IF_EMPTY,
    ),
    // Block customizations
    (".tendermint.types.BlockID.hash", HEXSTRING),
    (".tendermint.types.BlockID.part_set_header", RENAME_PARTS), // https://github.com/tendermint/tendermint/issues/5522
    (".tendermint.types.PartSetHeader.hash", HEXSTRING),
    (".tendermint.types.Header.height", FROM_STR),
    //(".tendermint.types.Header.time", ???), <- implement a serializer that converts from String to Option<Timestamp>
    // Let's implement these one-by-one for now. If it becomes cumbersome, we can return to relative paths.
    //("app_version", FROM_STR),
    //("round", FROM_STR),
    //("hash", HEXSTRING),
    //("app_hash", HEXSTRING),

    //("last_results_hash", HEXSTRING),
    //("last_commit_hash", HEXSTRING),
    //("data_hash", HEXSTRING),
    //("validators_hash", HEXSTRING),
    //("next_validators_hash", HEXSTRING),
    //("consensus_hash", HEXSTRING),
    //("evidence_hash", HEXSTRING),
];
