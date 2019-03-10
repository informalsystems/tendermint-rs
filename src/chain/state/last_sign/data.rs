use tendermint::block;

/// Last known chain state the last time we attempted to sign
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Data {
    pub height: i64,
    pub round: i64,
    pub step: i8,
    pub block_id: Option<block::Id>,
}
