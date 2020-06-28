use jsonrpc_core::futures::future::{self, Future, FutureResult};
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct State {
    header: String,
    commit: String,
    validator_set: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Status {
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

pub use self::rpc_impl_Rpc::gen_client::Client;

pub struct Server;

impl Rpc for Server {
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
    use pretty_assertions::assert_eq;

    use super::{Client, Rpc as _, Server};

    #[test]
    fn state() {
        let fut = {
            let mut io = IoHandler::new();
            io.extend_with(Server.to_delegate());
            let (client, server) = local::connect::<Client, _, _>(io);
            client.state().join(server)
        };
        let (res, _) = fut.wait().unwrap();

        assert_eq!(
            res,
            super::State {
                header: "".to_string(),
                commit: "".to_string(),
                validator_set: "".to_string(),
            }
        );
    }

    #[test]
    fn status() {
        let fut = {
            let mut io = IoHandler::new();
            io.extend_with(Server.to_delegate());
            let (client, server) = local::connect::<Client, _, _>(io);
            client.status().join(server)
        };
        let (res, _) = fut.wait().unwrap();

        assert_eq!(res, super::Status { latest_height: 12 });
    }
}
