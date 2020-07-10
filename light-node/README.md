# LightNode

Tendermint light client node wraps the light-client crate into a command-line interface tool. 
It can be used as a standalone light client daemon and exposes an RPC endpoint 
that exposes the current state of the light node. 

## Getting Started

To run the light node from source you have to clone this repository first:
```
$ git clone https://github.com/informalsystems/tendermint-rs.git
[...]
```

Then navigate to the light node crate:
```
$ cd tendermint-rs/light-node
```

### Configuration

You can configure all aspects of light node via a configuration file. 
An example cofigartion can be found under [light_node.toml.example](light-node/light_node.toml.example). 

If you are running a Tendermint fullnode on your machine, you can simply copy and use it to get started:
```
$ cp light_node.toml.example light_node.toml
``` 
Please, take a look into the config file and edit it according to your needs.

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

### Running the light node daemon

Now you can start your light node by simply running:
```
$ cargo run --  start                             
```

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

### RPC endpoint(s)

TODO

 