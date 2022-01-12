- `[tendermint-rpc]` The `event::Event::events` field is now represented as
  `Option<Vec<crate::abci::Event>>` as opposed to `Option<HashMap<String,
  Vec<String>>>` to accommodate breaking change in Tendermint v0.35.0
  subscription interface ([#862](https://github.com/informalsystems/tendermint-
  rs/issues/862))