- [`tendermint-light-client`] Remove the light client `Supervisor` API and
  associated data types. Users of the former supervisor are encouraged to
  instead call the light client directly via its `Instance` and `State` types.
  ([\#1291](https://github.com/informalsystems/tendermint-rs/issues/1291))
