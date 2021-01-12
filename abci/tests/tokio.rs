//! Tokio-based ABCI client/server interaction.

#[cfg(all(feature = "with-tokio", feature = "client", feature = "echo-app"))]
mod tokio_integration {
    use tendermint::abci::request::Echo;
    use tendermint_abci::{Client, EchoApp, TokioClient, TokioServer};

    #[tokio::test]
    async fn echo() {
        let app = EchoApp::new();
        let (server, term_tx) = TokioServer::bind("127.0.0.1:0", app).await.unwrap();
        let server_addr = server.local_addr();
        let server_handle = tokio::spawn(async move { server.listen().await });

        let mut client = TokioClient::connect(server_addr).await.unwrap();
        let requests = (0..5)
            .map(|r| Echo::new(format!("Request {}", r)))
            .collect::<Vec<Echo>>();
        for request in &requests {
            let res = client.echo(request.clone()).await.unwrap();
            assert_eq!(res.message, request.message);
        }

        term_tx.send(()).await.unwrap();
        server_handle.await.unwrap().unwrap();
    }
}
