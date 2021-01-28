//! `async-std`-based ABCI client/server integration tests.

#[cfg(all(
    feature = "non-blocking",
    feature = "runtime-async-std",
    feature = "client",
    feature = "echo-app"
))]
mod async_std_integration {
    use tendermint::abci::request;
    use tendermint_abci::runtime::non_blocking::Sender;
    use tendermint_abci::{AsyncStdClientBuilder, AsyncStdServerBuilder, EchoApp};

    #[async_std::test]
    async fn echo() {
        let requests = (0..5)
            .map(|r| request::Echo {
                message: format!("echo {}", r),
            })
            .collect::<Vec<request::Echo>>();
        let (server, term_tx) = AsyncStdServerBuilder::default()
            .bind("127.0.0.1:0", EchoApp::default())
            .await
            .unwrap();
        let server_addr = server.local_addr();
        let server_handle = async_std::task::spawn(async move { server.listen().await });

        let mut client = AsyncStdClientBuilder::default()
            .connect(server_addr)
            .await
            .unwrap();
        for req in requests {
            let res = client.echo(req.clone()).await.unwrap();
            assert_eq!(res.message, req.message);
        }

        term_tx.send(()).await.unwrap();
        server_handle.await.unwrap();
    }
}
