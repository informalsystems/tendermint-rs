//! Tokio-based ABCI client/server integration tests.

#[cfg(all(
    not(feature = "async"),
    feature = "runtime-std",
    feature = "client",
    feature = "echo-app"
))]
mod std_integration {
    use tendermint::abci::request;
    use tendermint_abci::{EchoApp, StdClient, StdServer};

    #[test]
    fn echo() {
        let requests = (0..5)
            .map(|r| request::Echo {
                message: format!("echo {}", r),
            })
            .collect::<Vec<request::Echo>>();
        let server = StdServer::bind("127.0.0.1:0", EchoApp::default()).unwrap();
        let server_addr = server.local_addr();
        let _ = std::thread::spawn(move || server.listen());

        let mut client = StdClient::connect(server_addr).unwrap();
        for req in requests {
            let res = client.echo(req.clone()).unwrap();
            assert_eq!(res.message, req.message);
        }
    }
}
