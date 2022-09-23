*Sep 23, 2022*

This release follows from v0.23.9, with the v0.24 series skipped due to
Tendermint Core [abandoning the v0.35 and v0.36
releases](https://github.com/informalsystems/tendermint-rs/discussions/1179). As
such, it is a non-breaking change, and removes the need to pin one's
tendermint-rs dependencies to a specific version (as was the case for the v0.23
series).

This release still targets compatibility with Tendermint Core v0.34, and
specifically provides compatibility with v0.34.21.
