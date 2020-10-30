# Tendermint RPC Probe

The Tendermint RPC probe is an application that assists in testing the various
crates in this repository. It currently allows you to execute a quick probe of
a running [Tendermint] node, where a quick probe executes requests against all
of the [Tendermint RPC] endpoints (including subscriptions for different event
types), and saves all of the responses it gets as JSON files. These JSON files
can be used in testing in other crates.

## Requirements

To run this probe locally, you will need:

* The Rust toolchain (latest stable)
* Docker

## Usage (with Docker)

From the root of the tendermint.rs repository:

```bash
cd rpc-probe
./run-with-docker.sh
```

This will:

1. Build the `tendermint-rpc-probe` executable
2. Pull the latest version of the Tendermint Docker image
3. Initialize and run a Tendermint node with the `kvstore` app in the
   background. (This node exposes a WebSocket endpoint at
   `ws://127.0.0.1:26657/websocket`)
4. Run `tendermint-rpc-probe` against the running Tendermint node.
5. Terminate the Docker image.

To run a specific version of Tendermint, simply:

```bash
TENDERMINT_TAG="v0.33.8" ./run-with-docker.sh
```

## Usage (without Docker)

Simply run:

```bash
cargo run -- --help
```

to see what options are available to run the probe.

For example:

```bash
# Executes the probe with all default options (i.e. against a Tendermint node
# listening on 127.0.0.1:26657)
cargo run

# Customize the address
cargo run -- --addr ws://192.168.1.15:26657/websocket

# Customize how long to wait before each request (in milliseconds)
# Defaults to 1000ms
cargo run -- --request-wait 100
```

## Output

By default, all request and response JSON-RPC messages will be written into a
folder called `probe-results` in the `rpc-probe` directory.

For example, the `probe-results/responses/abci_info.json` file (returned by the
[`abci_info`] RPC request) could look something like:

```json
{
  "id": "8944f639-7da0-4595-ac5e-3e432079f510",
  "jsonrpc": "2.0",
  "result": {
    "response": {
      "app_version": "1",
      "data": "{\"size\":0}",
      "last_block_app_hash": "AAAAAAAAAAA=",
      "last_block_height": "13",
      "version": "0.17.0"
    }
  }
}
```

The full JSON-RPC wrapper is saved to disk.

[Tendermint]: https://github.com/tendermint/tendermint
[Tendermint RPC]: https://docs.tendermint.com/master/rpc/
[`abci_info`]: https://docs.tendermint.com/master/rpc/#/ABCI/abci_info
