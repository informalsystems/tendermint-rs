- `[tendermint]` The `tendermint::node::info::ListenAddress::to_net_address`
  method was replaced with a simple `as_str` method toward facilitating
  `no_std` compatibility ([#983](https://github.com/informalsystems/tendermint-
  rs/issues/983))