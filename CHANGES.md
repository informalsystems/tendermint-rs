## [0.6.0-rc0] (2019-07-03)

This release is compatible with [tendermint v0.31].

### Upgrade Notes

#### `state_file` syntax changes

The validator state files use an incompatible syntax from Tendermint KMS v0.5.
It has been changed to match the conventions used by the rest of Tendermint,
where integer values are stored in strings rather than JSON integers.

When upgrading, you will need to either *delete existing state files* 
(they will be recreated automatically), or ensure the integer `height` and
`round` fields contained within these files are quoted in strings, e.g.
`{"height":"123456","round":"0",...}`.

#### Unknown fields now disallowed in `tmkms.toml`

The previous parser for `tmkms.toml` ignored unknown attributes in the
config file. This means it would often ignore syntax errors, spelling mistakes,
or attributes in the wrong location when parsing files.

This has been changed to explicitly reject such fields, however please be aware
if your config file contained invalid syntax, it will now be rejected by the
parser and the KMS will no longer boot.

We suggest validating the configuration in a staging or other noncritical
deployment of the KMS in order to ensure your configuration does not contain
accidental misconfigurations which were previously uncaught.

See [#282] for more information.

#### YubiHSM improvements

This release contains many improvements for users of the `yubihsm` backend:

- New `yubihsm-server` feature: this release includes support for the KMS
  exposing an HTTP service which is compatible with Yubico's
  `yubihsm-connector` service. This allows for concurrently administering
  a YubiHSM2 while the KMS is running, either through `tmkms yubihsm`
  (see additional notes below) or via Yubico's `yubihsm-shell`.
- Loopback support for `tmkms yubihsm`: the CLI functionality in the KMS for
  administering YubiHSMs can now be configured to connect to the KMS's
  own `yubihsm-server`. Additionally it can also be configured to use a
  different authentication key, and to prompt for a password as opposed to
  using one in the configuration file.

For more information on these changes, please see the "yubihsm-server feature"
section in the Tendermint KMS YubiHSM docs:

<https://github.com/tendermint/kms/blob/master/README.yubihsm.md>

### Detailed Changes

- [`abscissa` crate v0.1] ([#288])
- [`tendermint` crate v0.9] ([#280])
- Log durations for each signing operation ([#283])
- Add `serde(deny_unknown_fields)` to all config structs ([#282])
- `tmkms yubihsm keys list`: Use chain-specific formatters ([#275])
- `yubihsm-server`: Allow CLI commands to use loopback connection ([#274])
- `yubihsm-server`: Optional `yubihsm-connector` compatibility ([#273])
- Send `RemoteSignerError` response to validator on double sign ([#249])
- Logging improvements ([#271])
- yubihsm: Mark imported `priv_validator.json` keys as re-exportable ([#248])
- ledger: Add init commands ([#242])
- Add `max_height` support for stopping chains at specific heights ([#238])
- Chain-specific keyrings / multitenancy ([#232])
- ledger: Use `ledger-tendermint` backend ([#225])

## [0.5.0] (2019-03-13)

- [`tendermint` crate v0.5.0] ([#220])
- Optional peer ID verification ([#219])
- Bump subtle-encoding dependency to v0.3.3 ([#217])
- Allow setting config path via `TMKMS_CONFIG_FILE` env var ([#215])
- yubihsm: Add back HTTP connector support ([#208])
- Initial Tendermint `[chain]` registry in tmkms.toml ([#205])
- Disable 'softsign' backend by default ([#203])
- State tracking for double sign protection ([#193], thanks [@zmanian]!)

## [0.4.0] (2019-03-05)

- [`tendermint` crate v0.3.0] ([#200])
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

[0.6.0-rc0]: https://github.com/tendermint/kms/pull/289
[tendermint v0.31]: https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md#v0316
[`abscissa` crate v0.1]: https://github.com/iqlusioninc/abscissa/pull/77
[`tendermint` crate v0.9]: https://github.com/tendermint/kms/pull/280
[#288]: https://github.com/tendermint/kms/pull/288
[#283]: https://github.com/tendermint/kms/pull/283
[#282]: https://github.com/tendermint/kms/pull/282
[#280]: https://github.com/tendermint/kms/pull/280
[#275]: https://github.com/tendermint/kms/pull/275
[#274]: https://github.com/tendermint/kms/pull/274
[#273]: https://github.com/tendermint/kms/pull/273
[#249]: https://github.com/tendermint/kms/pull/249
[#271]: https://github.com/tendermint/kms/pull/271
[#248]: https://github.com/tendermint/kms/pull/248
[#242]: https://github.com/tendermint/kms/pull/242
[#238]: https://github.com/tendermint/kms/pull/238
[#232]: https://github.com/tendermint/kms/pull/232
[#225]: https://github.com/tendermint/kms/pull/225
[0.5.0]: https://github.com/tendermint/kms/pull/222
[`tendermint` crate v0.5.0]: https://crates.io/crates/tendermint/0.5.0
[#220]: https://github.com/tendermint/kms/pull/220
[#219]: https://github.com/tendermint/kms/pull/219
[#217]: https://github.com/tendermint/kms/pull/217
[#215]: https://github.com/tendermint/kms/pull/215
[#208]: https://github.com/tendermint/kms/pull/208
[#205]: https://github.com/tendermint/kms/pull/205
[#203]: https://github.com/tendermint/kms/pull/223
[#193]: https://github.com/tendermint/kms/pull/193
[@zmanian]: https://github.com/zmanian
[0.4.0]: https://github.com/tendermint/kms/pull/201
[`tendermint` crate v0.3.0]: https://crates.io/crates/tendermint/0.3.0
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
