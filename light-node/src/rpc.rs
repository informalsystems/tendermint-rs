use jsonrpc_core::futures::future::{self, Future, FutureResult};
use jsonrpc_core::Error;
use jsonrpc_derive::rpc;

#[rpc]
pub trait Rpc {
    /// Performs asynchronous operation
    #[rpc(name = "status")]
    fn status(&self) -> FutureResult<String, Error>;
}

struct RpcImpl;

impl Rpc for RpcImpl {
    fn status(&self) -> FutureResult<String, Error> {
        future::ok("OK".to_owned())
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
    fn hello() {
        let mut io = IoHandler::new();
        io.extend_with(RpcImpl.to_delegate());

        let fut = {
            let (client, server) = local::connect::<gen_client::Client, _, _>(io);
            client.status().map(|res| println!("{}", res)).join(server)
        };
        fut.wait().unwrap();
    }
}
