# Light-Node

The [Tendermint] light-node wraps the [light-client] crate into a command-line interface tool. 
It can be used as a standalone light client daemon and exposes a JSONRPC endpoint 
from which you can query the current state of the light node. 

## Getting Started

### Prerequisites

This short tutorial assumes that you are familiar with how to run a Tendermint fullnode on your machine. To learn how to do this, you can consult the [quick start] section of the tendermint documentation.

This tutorial further assumes you have `git` and the latest stable rust tool-chain installed (see https://rustup.rs/).
Additionally, the `jq` tool will make your life easier when dealing with JSON output.

#### Cloning the repository

To run the light node from source you have to clone this repository first:
```
$ git clone https://github.com/informalsystems/tendermint-rs.git
```

Then navigate to the light node crate:
```
$ cd tendermint-rs/light-node
```

### Configuration

You can configure all aspects of light node via a configuration file. 
An example cofigartion can be found under [light_node.toml.example](light_node.toml.example). 

If you are running a Tendermint fullnode on your machine, you can simply copy and use it to get started:
```
$ cp light_node.toml.example light_node.toml
``` 
Please, take a look into the config file and edit it according to your needs.
The provided example configuration file comes with a lot of explanatory comments
which hopefully provide enough guidance to configure your light node.

### Subjective initialization
Assuming that you are running a Tendermint fullnode that exposes an RPC endpoint on your loopback interface, you can intialize the light-node subjectively following th following steps:

First, you have to obtain a header hash and height you want to trust (subjectively). For our purposes you can obtain one via querying the Tendermint fullnode you are running. 
Here we are obtaining the header hash of height 2: 
```
$ curl -X GET "http://localhost:26657/block?height=2" -H  "accept: application/json" | jq .result.block_id.hash                                              1515:15:26
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  2155    0  2155    0     0   161k      0 --:--:-- --:--:-- --:--:--  161k
"76F85BEF1133114482FC8F78C5E78D2B1C1875DD8422A0394B175DD694A7FBA1"
```

You can now use this header hash to subjectively initialize your light node via:
```
$ cargo run --  initialize  2 76F85BEF1133114482FC8F78C5E78D2B1C1875DD8422A0394B175DD694A7FBA1
```

Note that calling `cargo run` for the first time might take a while as this command will also compile the light node and all its dependencies.

### Running the light node daemon

Now you can start your light node by simply running:
```
$ cargo run --  start                             
```

If everything worked the output will look sth like:
```
 cargo run --  start                           17:56:31
    Finished dev [unoptimized + debuginfo] target(s) in 0.42s
     Running `/redacted/tendermint-rs/target/debug/light_node start`
[info] synced to block 20041
[info] synced to block 20042
[info] synced to block 20044
[info] synced to block 20046
[info] synced to block 20048
[info] synced to block 20049
[info] synced to block 20051
[info] synced to block 20053
[info] synced to block 20054
[...]
```

You can stop the light node by pressing Ctrl+c.

### Help

You will notice that some config parameters can be overwritten via command line arguments. 

To get a full overview and commandline parameters and available sub-commands, run:

```
$ cargo run --  help
```
Or on a specific sub-command, e.g.:
 ```shell script
$ cargo run --  help start
 ```

### JSONRPC Endpoint(s)

When you have a light-node running you can query its current state via:
```
$ curl localhost:8888 -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc": "2.0", "method": "state", "id": 1}' | jq
```

<details>
  <summary><b>Click here</b> to see an example for expected output:</summary>

Command:  
  ```
$ curl localhost:8888 -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc": "2.0", "method": "state", "id": 1}' | jq 
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  1902  100  1856  100    46   164k   4181 --:--:-- --:--:-- --:--:--  168k
```
Example output:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "next_validator_set": {
      "validators": [
        {
          "address": "AD358F20C8CE80889E0F0248FDDC454595D632AE",
          "proposer_priority": "0",
          "pub_key": {
            "type": "tendermint/PubKeyEd25519",
            "value": "uo9rbgR5J0kuED0C529bTa6mcHZ4uXDjJRdg1k8proY="
          },
          "voting_power": "10"
        }
      ]
    },
    "provider": "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE",
    "signed_header": {
      "commit": {
        "block_id": {
          "hash": "76F85BEF1133114482FC8F78C5E78D2B1C1875DD8422A0394B175DD694A7FBA1",
          "parts": {
            "hash": "568F279E3F59FBE3CABEACE7A3C028C15CA6A902F9D77DDEBA3BFCB9514E2881",
            "total": "1"
          }
        },
        "height": "2",
        "round": "0",
        "signatures": [
          {
            "block_id_flag": 2,
            "signature": "sN3e6bzKLeIFNRptQ4SytBDLZJA53e92D6FWTll5Lq8Wdg4fVzxya6qx3SHFU82ukuj8jKmBMkwTTJsb8xThCQ==",
            "timestamp": "2020-07-10T12:39:06.977628900Z",
            "validator_address": "AD358F20C8CE80889E0F0248FDDC454595D632AE"
          }
        ]
      },
      "header": {
        "app_hash": "0000000000000000",
        "chain_id": "dockerchain",
        "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
        "data_hash": null,
        "evidence_hash": null,
        "height": "2",
        "last_block_id": {
          "hash": "F008EACA817CF6A3918CF7A6FD44F1F2464BB24D25A7EDB45A03E8783E9AB438",
          "parts": {
            "hash": "BF5130E879A02AC4BB83E392732ED4A37BE2F01304A615467EE7960858774E57",
            "total": "1"
          }
        },
        "last_commit_hash": "474496740A2EAA967EED02B239DA302BAF696AE36AEA78F7FEFCE4A77CCA5B33",
        "last_results_hash": null,
        "next_validators_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3",
        "proposer_address": "AD358F20C8CE80889E0F0248FDDC454595D632AE",
        "time": "2020-07-10T12:39:05.977628900Z",
        "validators_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3",
        "version": {
          "app": "1",
          "block": "10"
        }
      }
    },
    "validator_set": {
      "validators": [
        {
          "address": "AD358F20C8CE80889E0F0248FDDC454595D632AE",
          "proposer_priority": "0",
          "pub_key": {
            "type": "tendermint/PubKeyEd25519",
            "value": "uo9rbgR5J0kuED0C529bTa6mcHZ4uXDjJRdg1k8proY="
          },
          "voting_power": "10"
        }
      ]
    }
  },
  "id": 1
}

```

</details>


[quick start]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/quick-start.md
[Tendermint]: https://github.com/tendermint/tendermint
[light-client]: https://github.com/informalsystems/tendermint-rs/tree/master/light-client