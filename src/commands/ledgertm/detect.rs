use abscissa::Callable;

/// The `ledgertm detect` subcommand
#[derive(Debug, Default, Options)]
pub struct DetectCommand {
    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    pub verbose: bool,
}

impl Callable for DetectCommand {
    /// Detect all Ledger devices running the Tendermint app
    fn call(&self) {
        println!("This feature will be soon available");
    }
}
