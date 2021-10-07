- `[tendermint]` The `tendermint::node::info::OtherInfo::rpc_address`
  field type has been changed from `tendermint::net::Address`
  to `String` toward facilitating `no_std` compatibility
  ([#983](https://github.com/informalsystems/tendermint-rs/issues/983))