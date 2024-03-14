- Allow specifying a request timeout for the RPC `HttpClient`.
  `http::Builder` now provides a `.timeout(Duration)` method to specify the request timeout.
  If not specified, the default value is 30 seconds.
  ([\#1379](https://github.com/informalsystems/tendermint-rs/issues/1379))
