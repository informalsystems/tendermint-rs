use gumdrop::Options;

use tendermint_mbt_utils::commit::Commit;
use tendermint_mbt_utils::header::Header;
use tendermint_mbt_utils::producer::Producer;
use tendermint_mbt_utils::validator::Validator;
use simple_error::SimpleError;
use tendermint_mbt_utils::helpers::read_stdin;

const USAGE: &str = r#"
This is a small utility for producing tendermint datastructures
from minimal input (for testing purposes only).

For example, a tendermint validator can be produced only from an identifier,
or a tendermint header only from a set of validators.

To get an idea which input is needed for each datastructure, try '--help CMD':
it will list the required and optional parameters.

The parameters can be supplied in two ways:
  - via STDIN: in that case they are expected to be a valid JSON object,
    with each parameter being a field of this object
  - via command line arguments to the specific command.

If a parameter is supplied both via STDIN and CLI, the latter is given preference.

In case a particular datastructure can be produced from a single parameter
(like validator), there is a shortcut that allows to provide this parameter
directly via STDIN, without wrapping it into JSON object.
E.g., in the validator case, the following are equivalent:

    mbt-tendermint-produce validator --id a --voting-power 3
    echo -n '{"id": "a", "voting_power": 3}' | mbt-tendermint-produce --read-stdin validator
    echo -n a | mbt-tendermint-produce --read-stdin validator --voting-power 3
    echo -n '{"id": "a"}' | mbt-tendermint-produce --read-stdin validator --voting-power 3
    echo -n '{"id": "a", "voting_power": 100}' | mbt-tendermint-produce --read-stdin validator --voting-power 3

The result is:
    {
      "address": "730D3D6B2E9F4F0F23879458F2D02E0004F0F241",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "YnT69eNDaRaNU7teDTcyBedSD0B/Ziqx+sejm0wQba0="
      },
      "voting_power": "3",
      "proposer_priority": null
    }
"#;

#[derive(Debug, Options)]
struct CliOptions {
    #[options(help = "print this help and exit (--help CMD for command-specific help)")]
    help: bool,
    #[options(help = "provide detailed usage instructions")]
    usage: bool,
    #[options(help = "read input from STDIN (default: no)")]
    read_stdin: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "produce validator from identifier and other parameters")]
    Validator(Validator),
    #[options(help = "produce header from validator array and other parameters")]
    Header(Header),
    #[options(help = "produce commit from validator array and other parameters")]
    Commit(Commit),
}

fn encode_with_stdin<Opts: Producer<T> + Options, T: serde::Serialize>(cli: &Opts) -> Result<String, SimpleError>
{
    let stdin = read_stdin()?;
    let default = Opts::from_str(&stdin)?;
    let producer = cli.merge_with_default(&default);
    producer.encode()
}

fn run_command<Opts: Producer<T> + Options, T: serde::Serialize>(cli: Opts, read_stdin: bool) {
    let res = if read_stdin {
        encode_with_stdin(&cli)
    } else {
        cli.encode()
    };
    match res {
        Ok(res) => println!("{}", res),
        Err(e) => {
            println!("Error: {}\n", e);
            println!("Supported parameters for this command are: ");
            print_params(cli.self_usage())
        }
    }
}

fn print_params(options: &str) {
    for line in options.lines().skip(1) {
        eprintln!("{}", line);
    }
}

fn main() {
    let opts = CliOptions::parse_args_default_or_exit();
    if opts.usage {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    }
    match opts.command {
        None => {
            eprintln!("Produce tendermint datastructures for testing from minimal input\n");
            eprintln!("Please specify a command:");
            eprintln!("{}\n", CliOptions::command_list().unwrap());
            eprintln!("{}\n", CliOptions::usage());
            for cmd in CliOptions::command_list()
                .unwrap()
                .split('\n')
                .map(|s| s.split_whitespace().next().unwrap())
            {
                eprintln!("\n{} parameters:", cmd);
                print_params(CliOptions::command_usage(cmd).unwrap())
            }
            std::process::exit(1);
        }
        Some(Command::Validator(cli)) => run_command(cli, opts.read_stdin),
        Some(Command::Header(cli)) => run_command(cli, opts.read_stdin),
        Some(Command::Commit(cli)) => run_command(cli, opts.read_stdin),
    }
}
