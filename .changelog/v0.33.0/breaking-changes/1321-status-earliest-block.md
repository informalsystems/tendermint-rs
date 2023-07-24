- [`tendermint-rpc`] Decode the earliest block data fields of the `sync_info`
  object in `/status` response and expose them in the `SyncInfo` struct:
  `earliest_block_hash`, `earliest_app_hash`, `earliest_block_height`,
  `earliest_block_time`
  ([\#1321](https://github.com/informalsystems/tendermint-rs/pull/1321)).
