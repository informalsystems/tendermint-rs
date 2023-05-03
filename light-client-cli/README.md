[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]

See the [repo root] for build status, license, Rust version, etc.

# Light Client CLI

CLI for the Tendermint [Light Client][light-client].

## Usage

```
Usage: tendermint-light-client-cli [OPTIONS] --chain-id <CHAIN_ID> --primary <PRIMARY> --witnesses <WITNESSES> --trusted-height <TRUSTED_HEIGHT> --trusted-hash <TRUSTED_HASH>

Options:
      --chain-id <CHAIN_ID>
          Identifier of the chain
      --primary <PRIMARY>
          Primary RPC address
      --witnesses <WITNESSES>
          Comma-separated list of witnesses RPC addresses
      --trusted-height <TRUSTED_HEIGHT>
          Height of trusted header
      --trusted-hash <TRUSTED_HASH>
          Hash of trusted header
      --height <HEIGHT>
          Height of the header to verify
      --trust-threshold <TRUST_THRESHOLD>
          Trust threshold [default: 2/3]
      --trusting-period <TRUSTING_PERIOD>
          Trusting period, in seconds (default: two weeks) [default: 1209600]
      --max-clock-drift <MAX_CLOCK_DRIFT>
          Maximum clock drift, in seconds [default: 5]
      --max-block-lag <MAX_BLOCK_LAG>
          Maximum block lag, in seconds [default: 5]
  -v, --verbose...
          Increase verbosity, can be repeated up to 2 times
  -h, --help
          Print help
  -V, --version
          Print version
```


[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/tendermint-light-client-cli.svg
[crate-link]: https://crates.io/crates/tendermint-light-client-cli
[docs-image]: https://docs.rs/tendermint-light-client-cli/badge.svg
[docs-link]: https://docs.rs/tendermint-light-client-cli/

[//]: # (general links)

[repo root]: https://github.com/informalsystems/tendermint-rs
[light-client]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client
