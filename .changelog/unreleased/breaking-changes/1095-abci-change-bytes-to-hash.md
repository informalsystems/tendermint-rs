- `[tendermint]` Change hash fields' type from `Bytes`
  ([#1095](https://github.com/informalsystems/tendermint-rs/issues/1095)):

  | Struct                         | Field                 | Type      |
  | ------------------------------ | --------------------- | --------- |
  | `abci::request::OfferSnapshot` | `app_hash`            | `AppHash` |
  | `abci::response::Info`         | `last_block_app_hash` | `AppHash` |
  | `abci::response::InitChain`    | `app_hash`            | `AppHash` |
  | `Genesis`                      | `app_hash`            | `AppHash` |

- `[tendermint]` Remove method `AppHash::value`,
  replaced with non-allocating `AppHash::as_bytes`
  [#1232](https://github.com/informalsystems/tendermint-rs/pull/1232).
