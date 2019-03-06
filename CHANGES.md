## [0.4.0] (2019-03-05)

- [`tendermint` crate v0.3.0](https://crates.io/crates/tendermint/0.3.0) ([#200])
- yubihsm: Support for exporting/importing wrapped (encrypted) keys ([#197])
- yubihsm setup ([#180], [#186])
- Ledger integration ([#176])

## [0.3.0] (2019-01-23)

- Add ability to terminate on SIGTERM or SIGINT ([#161])
- Remove `PoisonPillMsg` ([#162]) 

## [0.2.4] (2019-01-18)

- Refactor client/tests to always dial out to tendermint/gaiad ([#149], [#150])
- Migrate to rust 2018 edition ([#138])

## [0.2.3] (2018-12-08)

- Lower reconnect delay to 1s ([#136])

## [0.2.2] (2018-12-03)

- Allow empty BlockIds in validation method ([#131], [#132])

## [0.2.1] (2018-11-27)

- Encode node (and softwign) private keys as Base64 ([#127])
- Add integration tests for yubihsm subcommands ([#121])
- Fix `tmkms yubihsm keys import` command ([#113])

## [0.2.0] (2018-11-20)

- Add `tmkms yubihsm keys import` command ([#107])
- Simplify `tmkms.toml` syntax ([#106])
- Minor clarifications/fixes ([#103])

## [0.1.0] (2018-11-13)

- Initial validator signing support ([#95], [#91], [#86], [#80], [#55])
- Extract `tendermint` crate as a reusable Rust library ([#82])
- Support for Bech32-formatted Cosmos keys/addresses ([#71])
- Validator signing via Unix domain socket IPC ([#63])

## 0.0.1 (2018-10-16)

- Initial "preview" release

[0.4.0]: https://github.com/tendermint/kms/pull/201
[#200]: https://github.com/tendermint/kms/pull/200
[#197]: https://github.com/tendermint/kms/pull/197
[#186]: https://github.com/tendermint/kms/pull/186
[#180]: https://github.com/tendermint/kms/pull/180
[#176]: https://github.com/tendermint/kms/pull/176
[0.3.0]: https://github.com/tendermint/kms/pull/165
[#161]: https://github.com/tendermint/kms/pull/161
[#162]: https://github.com/tendermint/kms/pull/162
[0.2.4]: https://github.com/tendermint/kms/pull/156
[#149]: https://github.com/tendermint/kms/pull/149
[#150]: https://github.com/tendermint/kms/pull/150
[#138]: https://github.com/tendermint/kms/pull/138
[0.2.3]: https://github.com/tendermint/kms/pull/137
[#136]: https://github.com/tendermint/kms/pull/136
[0.2.2]: https://github.com/tendermint/kms/pull/134
[#132]: https://github.com/tendermint/kms/pull/132
[#131]: https://github.com/tendermint/kms/pull/131
[0.2.1]: https://github.com/tendermint/kms/pull/126
[#127]: https://github.com/tendermint/kms/pull/127
[#121]: https://github.com/tendermint/kms/pull/121
[#113]: https://github.com/tendermint/kms/pull/113
[0.2.0]: https://github.com/tendermint/kms/pull/108
[#107]: https://github.com/tendermint/kms/pull/107
[#106]: https://github.com/tendermint/kms/pull/106
[#103]: https://github.com/tendermint/kms/pull/103
[0.1.0]: https://github.com/tendermint/kms/pull/100
[#95]: https://github.com/tendermint/kms/pull/95
[#91]: https://github.com/tendermint/kms/pull/91
[#86]: https://github.com/tendermint/kms/pull/86
[#82]: https://github.com/tendermint/kms/pull/82
[#80]: https://github.com/tendermint/kms/pull/80
[#71]: https://github.com/tendermint/kms/pull/71
[#63]: https://github.com/tendermint/kms/pull/63
[#55]: https://github.com/tendermint/kms/pull/55
