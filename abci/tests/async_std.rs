/// `async-std`-based ABCI client/server interaction.

#[cfg(all(feature = "with-async-std", feature = "client", feature = "echo-app"))]
mod async_std_integration {
    use tendermint::abci::request::Echo;
    use tendermint_abci::{AsyncStdClient, AsyncStdServer, Client, EchoApp};

    #[async_std::test]
    async fn echo() {
        let app = EchoApp::new();
        let (server, term_tx) = AsyncStdServer::bind("127.0.0.1:0", app).await.unwrap();
        let server_addr = server.local_addr();
        let server_handle = async_std::task::spawn(async move { server.listen().await });

        let mut client = AsyncStdClient::connect(server_addr).await.unwrap();
        let requests = (0..5)
            .map(|r| Echo::new(format!("Request {}", r)))
            .collect::<Vec<Echo>>();
        for request in &requests {
            let res = client.echo(request.clone()).await.unwrap();
            assert_eq!(res.message, request.message);
        }

        term_tx.send(()).await.unwrap();
        server_handle.await.unwrap();
    }
}
