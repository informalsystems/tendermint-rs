use jsonrpc_core::futures::future::{self, Future, FutureResult};
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
struct State {
    header: String,
    commit: String,
    validator_set: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Status {
    latest_height: u64,
}

#[rpc]
pub trait Rpc {
    /// Latest state.
    #[rpc(name = "state")]
    fn state(&self) -> FutureResult<State, Error>;

    /// TODO(xla): Document.
    #[rpc(name = "status")]
    fn status(&self) -> FutureResult<Status, Error>;
}

struct RpcImpl;

impl Rpc for RpcImpl {
    fn state(&self) -> FutureResult<State, Error> {
        future::ok(State::default())
    }

    fn status(&self) -> FutureResult<Status, Error> {
        future::ok(Status { latest_height: 12 })
    }
}

#[cfg(test)]
mod test {
    use jsonrpc_core::futures::future::Future;
    use jsonrpc_core::IoHandler;
    use jsonrpc_core_client::transports::local;

    use super::rpc_impl_Rpc::gen_client;
    use super::rpc_impl_Rpc::gen_server::Rpc;
    use super::RpcImpl;

    #[test]
    fn state() {
        let mut io = IoHandler::new();
        io.extend_with(RpcImpl.to_delegate());

        let fut = {
            let (client, server) = local::connect::<gen_client::Client, _, _>(io);
            client.state().map(|res| println!("{:?}", res)).join(server)
        };
        fut.wait().unwrap();
    }

    #[test]
    fn status() {
        let mut io = IoHandler::new();
        io.extend_with(RpcImpl.to_delegate());

        let fut = {
            let (client, server) = local::connect::<gen_client::Client, _, _>(io);
            client
                .status()
                .map(|res| println!("{:?}", res))
                .join(server)
        };
        fut.wait().unwrap();
    }
}
