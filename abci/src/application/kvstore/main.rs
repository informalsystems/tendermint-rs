//! In-memory key/value store application for Tendermint.

use tendermint_abci::{KeyValueStoreApp, ServerBuilder};

fn main() {
    env_logger::init();

    let (app, driver) = KeyValueStoreApp::new();
    let server = ServerBuilder::default()
        .bind("127.0.0.1:26658", app)
        .unwrap();
    std::thread::spawn(move || driver.run());
    server.listen().unwrap();
}
