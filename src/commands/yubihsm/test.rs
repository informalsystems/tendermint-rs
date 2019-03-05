use abscissa::Callable;
use std::{
    thread,
    time::{Duration, Instant},
};

// TODO: figure out rough size of the proposal amino message for testing
const TEST_MESSAGE: &[u8; 128] = &[0u8; 128];

/// The `yubihsm test` subcommand
#[derive(Debug, Default, Options)]
pub struct TestCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Key ID to use for test
    #[options(free)]
    key_id: u16,
}

impl Callable for TestCommand {
    /// Perform a signing test using the current HSM configuration
    fn call(&self) {
        let hsm = crate::yubihsm::client();

        loop {
            let started_at = Instant::now();

            if let Err(e) = hsm.sign_ed25519(self.key_id, TEST_MESSAGE.as_ref()) {
                status_err!("signature operation failed: {}", e);
                thread::sleep(Duration::from_millis(250));
            } else {
                let duration = Instant::now().duration_since(started_at);
                status_ok!(
                    "Success",
                    "signed message using key ID #{} in {} ms",
                    self.key_id,
                    duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
                );
            }
        }
    }
}
