/// Tendermint protobuf version

/// Information on a Tendermint snapshot to generate prost structures from.
pub struct TendermintVersion {
    /// Repository URL.
    pub repo: &'static str,
    /// A commitish reference in the tendermint git repository, for example:
    ///
    /// - Tag: `v0.34.0-rc4`
    /// - Branch: `main`
    /// - Commit ID (full length): `d7d0ffea13c60c98b812d243ba5a2c375f341c15`
    pub commitish: &'static str,
}

/// All Tendermint versions to generate code for
pub const TENDERMINT_VERSION: TendermintVersion = TendermintVersion {
    repo: "https://github.com/cometbft/cometbft",
    commitish: "mikhail/proto-version-suffixes",
};

/// Predefined custom attributes for message annotations
const PRIMITIVE_ENUM: &str = r#"#[derive(::num_derive::FromPrimitive, ::num_derive::ToPrimitive)]"#;
const SERIALIZED: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;
const TYPE_TAG: &str = r#"#[serde(tag = "type", content = "value")]"#;

/// Predefined custom attributes for field annotations
const QUOTED: &str = r#"#[serde(with = "crate::serializers::from_str")]"#;
const QUOTED_WITH_DEFAULT: &str = r#"#[serde(with = "crate::serializers::from_str", default)]"#;
const DEFAULT: &str = r#"#[serde(default)]"#;
const HEXSTRING: &str = r#"#[serde(with = "crate::serializers::bytes::hexstring")]"#;
const BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::base64string")]"#;
const VEC_BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::vec_base64string")]"#;
const OPTIONAL: &str = r#"#[serde(with = "crate::serializers::optional")]"#;
const BYTES_SKIP_IF_EMPTY: &str = r#"#[serde(skip_serializing_if = "bytes::Bytes::is_empty")]"#;
const NULLABLEVECARRAY: &str = r#"#[serde(with = "crate::serializers::txs")]"#;
const NULLABLE: &str = r#"#[serde(with = "crate::serializers::nullable")]"#;
const ALIAS_POWER_QUOTED: &str =
    r#"#[serde(alias = "power", with = "crate::serializers::from_str")]"#;
const PART_SET_HEADER_TOTAL: &str =
    r#"#[serde(with = "crate::serializers::part_set_header_total")]"#;
const RENAME_EDPUBKEY: &str = r#"#[serde(rename = "tendermint/PubKeyEd25519", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_SECPPUBKEY: &str = r#"#[serde(rename = "tendermint/PubKeySecp256k1", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_SRPUBKEY: &str = r#"#[serde(rename = "tendermint/PubKeySr25519", with = "crate::serializers::bytes::base64string")]"#;
const RENAME_DUPLICATEVOTE: &str = r#"#[serde(rename = "tendermint/DuplicateVoteEvidence")]"#;
const RENAME_LIGHTCLIENTATTACK: &str =
    r#"#[serde(rename = "tendermint/LightClientAttackEvidence")]"#;
const ALIAS_VALIDATOR_POWER_QUOTED: &str =
    r#"#[serde(alias = "ValidatorPower", with = "crate::serializers::from_str")]"#;
const ALIAS_TOTAL_VOTING_POWER_QUOTED: &str =
    r#"#[serde(alias = "TotalVotingPower", with = "crate::serializers::from_str")]"#;
const ALIAS_TIMESTAMP: &str = r#"#[serde(alias = "Timestamp")]"#;
const ALIAS_PARTS: &str = r#"#[serde(alias = "parts")]"#;

/// Custom type attributes applied on top of protobuf structs
/// The first item in the tuple defines the message where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".cometbft.abci.v1.ResponseInfo", SERIALIZED),
    (".cometbft.crypto.v1.Proof", SERIALIZED),
    (".cometbft.crypto.v1.PublicKey", SERIALIZED),
    (".cometbft.crypto.v1.PublicKey.sum", TYPE_TAG),
    (".cometbft.libs.bits.v1.BitArray", SERIALIZED),
    (".cometbft.types.v3.Block", SERIALIZED),
    (".cometbft.types.v1.BlockID", SERIALIZED),
    (".cometbft.types.v1.BlockIDFlag", PRIMITIVE_ENUM),
    (".cometbft.types.v1.BlockMeta", SERIALIZED),
    (".cometbft.types.v1.CanonicalBlockID", SERIALIZED),
    (".cometbft.types.v1.CanonicalPartSetHeader", SERIALIZED),
    (".cometbft.types.v1.CanonicalVote", SERIALIZED),
    (".cometbft.types.v1.Commit", SERIALIZED),
    (".cometbft.types.v1.CommitSig", SERIALIZED),
    (".cometbft.types.v1.Data", SERIALIZED),
    (".cometbft.types.v3.DuplicateVoteEvidence", SERIALIZED),
    (".cometbft.types.v3.Evidence.sum", SERIALIZED),
    (".cometbft.types.v3.Evidence.sum", TYPE_TAG),
    (".cometbft.types.v3.EvidenceList", SERIALIZED),
    (".cometbft.types.v1.EvidenceParams", SERIALIZED),
    (".cometbft.types.v1.Header", SERIALIZED),
    (".cometbft.types.v1.LightClientAttackEvidence", SERIALIZED),
    (".cometbft.types.v1.LightBlock", SERIALIZED),
    (".cometbft.types.v1.PartSetHeader", SERIALIZED),
    (".cometbft.types.v1.SignedHeader", SERIALIZED),
    (".cometbft.types.v1.TxProof", SERIALIZED),
    (".cometbft.types.v1.Validator", SERIALIZED),
    (".cometbft.types.v1.ValidatorSet", SERIALIZED),
    (".cometbft.types.v3.Vote", SERIALIZED),
    (".cometbft.version.v1.Consensus", SERIALIZED),
];

/// Custom field attributes applied on top of protobuf fields in (a) struct(s)
/// The first item in the tuple defines the field where the annotation should apply and
/// the second item is the string that should be added as annotation.
/// The first item is a path as defined in the prost_build::Config::btree_map here:
/// https://docs.rs/prost-build/0.6.1/prost_build/struct.Config.html#method.btree_map
pub static CUSTOM_FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    (".cometbft.abci.v1.ResponseInfo.data", DEFAULT),
    (".cometbft.abci.v1.ResponseInfo.version", DEFAULT),
    (
        ".cometbft.abci.v1.ResponseInfo.app_version",
        QUOTED_WITH_DEFAULT,
    ),
    (
        ".cometbft.abci.v1.ResponseInfo.last_block_height",
        QUOTED_WITH_DEFAULT,
    ),
    (
        ".cometbft.abci.v1.ResponseInfo.last_block_app_hash",
        DEFAULT,
    ),
    (
        ".cometbft.abci.v1.ResponseInfo.last_block_app_hash",
        BYTES_SKIP_IF_EMPTY,
    ),
    (".cometbft.crypto.v1.Proof.index", QUOTED),
    (".cometbft.crypto.v1.Proof.total", QUOTED),
    (".cometbft.crypto.v1.Proof.aunts", VEC_BASE64STRING),
    (".cometbft.crypto.v1.Proof.leaf_hash", BASE64STRING),
    (".cometbft.crypto.v1.PublicKey.sum.ed25519", RENAME_EDPUBKEY),
    (
        ".cometbft.crypto.v1.PublicKey.sum.secp256k1",
        RENAME_SECPPUBKEY,
    ),
    (".cometbft.crypto.v1.PublicKey.sum.sr25519", RENAME_SRPUBKEY),
    (".cometbft.types.v1.BlockID.hash", HEXSTRING),
    (".cometbft.types.v1.BlockID.part_set_header", ALIAS_PARTS),
    (".cometbft.types.v1.BlockMeta.block_size", QUOTED),
    (".cometbft.types.v1.BlockMeta.num_txs", QUOTED),
    (
        ".cometbft.types.v1.CanonicalBlockID.part_set_header",
        ALIAS_PARTS,
    ),
    (".cometbft.types.v1.Commit.height", QUOTED),
    (".cometbft.types.v1.Commit.signatures", NULLABLE),
    (".cometbft.types.v1.CommitSig.validator_address", HEXSTRING),
    (".cometbft.types.v1.CommitSig.timestamp", OPTIONAL),
    (".cometbft.types.v1.CommitSig.signature", BASE64STRING),
    (".cometbft.types.v1.Data.txs", NULLABLEVECARRAY),
    (
        ".cometbft.types.v3.DuplicateVoteEvidence.total_voting_power",
        ALIAS_TOTAL_VOTING_POWER_QUOTED,
    ),
    (
        ".cometbft.types.v3.DuplicateVoteEvidence.validator_power",
        ALIAS_VALIDATOR_POWER_QUOTED,
    ),
    (
        ".cometbft.types.v3.DuplicateVoteEvidence.timestamp",
        ALIAS_TIMESTAMP,
    ),
    (
        ".cometbft.types.v3.Evidence.sum.duplicate_vote_evidence",
        RENAME_DUPLICATEVOTE,
    ),
    (
        ".cometbft.types.v3.Evidence.sum.light_client_attack_evidence",
        RENAME_LIGHTCLIENTATTACK,
    ),
    (".cometbft.types.v3.EvidenceList.evidence", NULLABLE),
    (
        ".cometbft.types.v1.EvidenceParams.max_bytes",
        QUOTED_WITH_DEFAULT,
    ),
    (".cometbft.types.v1.Header.height", QUOTED),
    (".cometbft.types.v1.Header.time", OPTIONAL),
    (".cometbft.types.v1.Header.last_commit_hash", HEXSTRING),
    (".cometbft.types.v1.Header.data_hash", HEXSTRING),
    (".cometbft.types.v1.Header.validators_hash", HEXSTRING),
    (".cometbft.types.v1.Header.next_validators_hash", HEXSTRING),
    (".cometbft.types.v1.Header.consensus_hash", HEXSTRING),
    (".cometbft.types.v1.Header.app_hash", HEXSTRING),
    (".cometbft.types.v1.Header.last_results_hash", HEXSTRING),
    (".cometbft.types.v1.Header.evidence_hash", HEXSTRING),
    (".cometbft.types.v1.Header.proposer_address", HEXSTRING),
    (
        ".cometbft.types.v1.PartSetHeader.total",
        PART_SET_HEADER_TOTAL,
    ),
    (".cometbft.types.v1.PartSetHeader.hash", HEXSTRING),
    (".cometbft.types.v1.TxProof.data", BASE64STRING),
    (".cometbft.types.v1.TxProof.root_hash", HEXSTRING),
    (".cometbft.types.v1.Validator.address", HEXSTRING),
    (
        ".cometbft.types.v1.Validator.voting_power",
        ALIAS_POWER_QUOTED,
    ), // https://github.com/tendermint/tendermint/issues/5549
    (
        ".cometbft.types.v1.Validator.proposer_priority",
        QUOTED_WITH_DEFAULT,
    ), // Default is for /genesis deserialization
    (".cometbft.types.v3.Vote.height", QUOTED),
    (".cometbft.types.v3.Vote.validator_address", HEXSTRING),
    (".cometbft.types.v3.Vote.signature", BASE64STRING),
    (".cometbft.types.v3.Vote.timestamp", OPTIONAL),
    (".cometbft.version.v1.Consensus.block", QUOTED),
    (".cometbft.version.v1.Consensus.app", QUOTED_WITH_DEFAULT),
];
