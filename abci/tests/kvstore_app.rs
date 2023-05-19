//! Key/value store application integration tests.

#[cfg(all(feature = "client", feature = "kvstore-app"))]
mod kvstore_app_integration {
    use std::thread;

    use tendermint_abci::{ClientBuilder, KeyValueStoreApp, ServerBuilder};
    use tendermint_proto::v0_38::abci::{RequestEcho, RequestFinalizeBlock, RequestQuery};

    #[test]
    fn happy_path() {
        let (app, driver) = KeyValueStoreApp::new();
        let server = ServerBuilder::default().bind("127.0.0.1:0", app).unwrap();
        let server_addr = server.local_addr();
        thread::spawn(move || driver.run());
        thread::spawn(move || server.listen());

        let mut client = ClientBuilder::default().connect(server_addr).unwrap();
        let res = client
            .echo(RequestEcho {
                message: "Hello ABCI!".to_string(),
            })
            .unwrap();
        assert_eq!(res.message, "Hello ABCI!");

        client
            .finalize_block(RequestFinalizeBlock {
                txs: vec!["test-key=test-value".into()],
                ..Default::default()
            })
            .unwrap();
        client.commit().unwrap();

        let res = client
            .query(RequestQuery {
                data: "test-key".into(),
                path: "".to_string(),
                height: 0,
                prove: false,
            })
            .unwrap();
        assert_eq!(res.value, "test-value".as_bytes());
    }
}
