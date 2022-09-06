## tendermint-testgen

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]

`tendermint-testgen` is a small utility for producing Tendermint data
structures from minimal input (for testing purposes only).

## Requirements

- Latest Rust stable

## Usage

```bash
# Show usage information for tendermint-testgen
cargo run -- --help

# Show usage about a particular command
cargo run -- --help CMD
```

As an example, a Tendermint validator can be produced only from an identifier,
or a Tendermint header only from a set of validators.

The parameters can be supplied in two ways:
  - via STDIN: in that case they are expected to be a valid JSON object,
    with each parameter being a field of this object
  - via command line arguments to the specific command.

If a parameter is supplied both via STDIN and CLI, the latter is given preference.

In case a particular data structure can be produced from a single parameter
(like validator), there is a shortcut that allows to provide this parameter
directly via STDIN, without wrapping it into JSON object.
E.g., in the validator case, the following commands are all equivalent:

```bash
tendermint-testgen validator --id a --voting-power 3
echo -n '{"id": "a", "voting_power": 3}' | tendermint-testgen --stdin validator
echo -n a | tendermint-testgen --stdin validator --voting-power 3
echo -n '{"id": "a"}' | tendermint-testgen --stdin validator --voting-power 3
echo -n '{"id": "a", "voting_power": 100}' | tendermint-testgen --stdin validator --voting-power 3
```

The result is:

```json
{
  "address": "730D3D6B2E9F4F0F23879458F2D02E0004F0F241",
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "YnT69eNDaRaNU7teDTcyBedSD0B/Ziqx+sejm0wQba0="
  },
  "voting_power": "3",
  "proposer_priority": null
}
```

## License

Copyright © 2020 Informal Systems

Licensed under the Apache License, Version 2.0 (the "License");
you may not use the files in this repository except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/tendermint-testgen.svg
[crate-link]: https://crates.io/crates/tendermint-testgen
[docs-image]: https://docs.rs/tendermint-testgen/badge.svg
[docs-link]: https://docs.rs/tendermint-testgen/
[build-image]: https://github.com/informalsystems/tendermint-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3ARust
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/informalsystems/tendermint-rs/blob/main/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg

[//]: # (general links)
