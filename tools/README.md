# Tools
This folder contains tools that we use during development and testing.

## kvstore-test
This crate allows the developers to do integration testing against a Tendermint Go endpoint. Our CI also uses it.

To run the tests locally, provided that a kvstore RPC endpoint is running on http://127.0.0.1:26657:
```shell
cargo test
```

Alternatively, you can run:
```shell
cargo test-all-features
```
which is exactly what we run in CI.

If you don't have an endpoint running, but you have Docker installed, you can ask the testing framework to fire up a
Docker container with the current stable Tendermint node. This happens automatically if you run:
```shell
cargo make
```

and all tests will run while the docker container is available. As additional help, you can run

```shell
cargo make docker-up
```

and

```shell
cargo make docker-down
```

to manage the Docker container.
