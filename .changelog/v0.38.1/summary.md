*July 23rd, 2024*

This release enhances `/block_results` response handling by trying to decode it as base64 if it fails to decode as hex.
This release also sets the versions of `prost` and `prost-types` their latest versions in the `tendermint` crate.