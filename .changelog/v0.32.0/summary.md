This release notably comes with a fully featured [light client attack detector][attack-detector],
and introduces a [CLI for the light client][light-client-cli] for verifying headers,
detecting attacks against the light client, and reporting the evidence to primary and witness nodes.

It also adds a [`Verifier::verify_misbehaviour_header`][verifier-method] method for verifying
headers coming from a misbehaviour evidence.

Moreover, the [`Client`][client-trait] trait is now exposed by the `tendermint-rpc` without requiring
the `http-client` or the `websocket-client` feature flags to be enabled.

[light-client-cli]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client-cli
[attack-detector]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client-detector
[verifier-method]: https://github.com/informalsystems/tendermint-rs/blob/6a4cd245b6f362832b974104b40be973dd0ef108/light-client-verifier/src/verifier.rs#L67
[client-trait]: https://github.com/informalsystems/tendermint-rs/blob/6a4cd245b6f362832b974104b40be973dd0ef108/rpc/src/client.rs#L49
