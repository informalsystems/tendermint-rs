//! Tokio-based ABCI client/server interaction.

#[cfg(feature = "with-tokio")]
mod integration {
    use tendermint::abci::request::Echo;
    use tendermint_abci::client::tokio::TokioClient;
    use tendermint_abci::client::Client;
    use tendermint_abci::server::tokio::TokioServer;
    use tendermint_abci::Application;

    /// Simple echo application for use in testing.
    #[derive(Clone)]
    pub struct EchoApp {}

    impl Default for EchoApp {
        fn default() -> Self {
            Self {}
        }
    }

    impl Application for EchoApp {}

    #[tokio::test]
    async fn echo() {
        let app = EchoApp::default();
        let (server, term_tx) = TokioServer::bind("127.0.0.1:0", app).await.unwrap();
        let server_addr = server.local_addr();
        let server_handle = tokio::spawn(async move { server.listen().await });

        let mut client = TokioClient::connect(server_addr).await.unwrap();
        let res = client
            .echo(Echo::new("Hello ABCI!".to_owned()))
            .await
            .unwrap();
        assert_eq!(res.message, "Hello ABCI!");

        term_tx.send(()).await.unwrap();
        server_handle.await.unwrap().unwrap();
    }
}
