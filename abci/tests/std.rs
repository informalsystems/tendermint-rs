//! Rust standard library-based ABCI client/server integration tests.

#[cfg(all(
    feature = "blocking",
    feature = "runtime-std",
    feature = "client",
    feature = "echo-app"
))]
mod std_integration {
    use tendermint::abci::request;
    use tendermint_abci::{EchoApp, StdClientBuilder, StdServerBuilder};

    #[test]
    fn echo() {
        let requests = (0..5)
            .map(|r| request::Echo {
                message: format!("echo {}", r),
            })
            .collect::<Vec<request::Echo>>();
        let server = StdServerBuilder::default()
            .bind("127.0.0.1:0", EchoApp::default())
            .unwrap();
        let server_addr = server.local_addr();
        let _ = std::thread::spawn(move || server.listen());

        let mut client = StdClientBuilder::default().connect(server_addr).unwrap();
        for req in requests {
            let res = client.echo(req.clone()).unwrap();
            assert_eq!(res.message, req.message);
        }
    }
}
