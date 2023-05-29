- `[tendermint-rpc]`  Changes to support the RPC protocol in CometBFT 0.38
  ([\#1317](https://github.com/informalsystems/tendermint-rs/pull/1317)):
  * Add `finalize_block_results` and `app_hash` fields to
    `endpoint::block_results::Response`.
  * The `deliver_tx` field is renamed to `tx_result` in
    `endpoint::broadcast::tx_commit::Response`.
  * The `tx_result` field type changed to `ExecTxResult` in
    `endpoint::tx::Response`.
  * The `event::EventData::NewBlock` variant is renamed to `LegacyNewBlock`.
    The new `NewBlock` variant only carries fields relevant since CometBFT 0.38.
  * Removed `event::DialectEvent`, replaced with `event::v0_34::DialectEvent`
    and `event::latest::DialectEvent` as non-generic serialization helpers.
    The latter handles the fields added in CometBFT 0.38, `block_id` and
    `result_finalize_block`. Same refactoring done for `DialectEventData`
    and other types used in the event data structures.
  * Changed some of the serialization dialect helpers only be
    used by the 0.34 dialect and remove generics. The current dialect's
    seralization is switched to the serde impls on the domain types in
    `tendermint`.
- `[tendermint]` Changes to support the RPC protocol in CometBFT 0.38
  ([\#1317](https://github.com/informalsystems/tendermint-rs/pull/1317)):
  * Due to some attribute changes, the format emitted by `Serialize` is
    changed for `abci::response` types `CheckTx` and `FinalizeBlock`.