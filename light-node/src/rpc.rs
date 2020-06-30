use jsonrpc_core::futures::future::{self, Future, FutureResult};
use jsonrpc_core::Error as jsonrpcError;
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

use tendermint_light_client::errors::Error;
use tendermint_light_client::supervisor::Handle;
use tendermint_light_client::types::LightBlock;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Status {
    latest_height: u64,
}

#[rpc]
pub trait Rpc {
    /// Latest state.
    #[rpc(name = "state")]
    fn state(&self) -> FutureResult<Option<LightBlock>, Error>;

    /// TODO(xla): Document.
    #[rpc(name = "status")]
    fn status(&self) -> FutureResult<Status, jsonrpcError>;
}

pub use self::rpc_impl_Rpc::gen_client::Client;

pub struct Server {
    handle: Handle,
}

impl Server {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }
}

impl Rpc for Server {
    fn state(&self) -> FutureResult<Option<LightBlock>, Error> {
        future::result(self.handle.latest_trusted())
    }

    fn status(&self) -> FutureResult<Status, jsonrpcError> {
        future::ok(Status { latest_height: 12 })
    }
}

#[cfg(test)]
mod test {
    use futures::compat::Future01CompatExt as _;
    use jsonrpc_core::futures::future::Future;
    use jsonrpc_core::IoHandler;
    use jsonrpc_core_client::transports::local;
    use pretty_assertions::assert_eq;

    use super::{Client, Rpc as _, Server};

    #[tokio::test]
    async fn state() {
        let server = Server::new();
        let fut = {
            let mut io = IoHandler::new();
            io.extend_with(server.to_delegate());
            let (client, server) = local::connect::<Client, _, _>(io);
            client.state().join(server)
        };
        let (res, _) = fut.compat().await.unwrap();

        assert_eq!(
            res,
            super::State {
                header: "".to_string(),
                commit: "".to_string(),
                validator_set: "".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn status() {
        let server = Server::new();
        let fut = {
            let mut io = IoHandler::new();
            io.extend_with(server.to_delegate());
            let (client, server) = local::connect::<Client, _, _>(io);
            client.status().join(server)
        };
        let (res, _) = fut.compat().await.unwrap();

        assert_eq!(res, super::Status { latest_height: 12 });
    }
}
