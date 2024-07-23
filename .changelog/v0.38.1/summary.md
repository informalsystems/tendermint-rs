*July 23rd, 2024*

This release enhances decoding of the `AppHash` type by trying to decode it as base64 if it fails to decode as hex.
This release also updates `prost` and `prost-types` to their latest version in the `tendermint` crate, something that was missed in the v0.38.0 release.