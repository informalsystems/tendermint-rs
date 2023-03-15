use tendermint_rpc::HttpClient;

use crate::instance::Instance;

#[derive(Debug)]
pub struct Peer {
    pub instance: Instance,
    pub rpc_client: HttpClient,
}

impl Peer {
    pub fn new(instance: Instance, rpc_client: HttpClient) -> Self {
        Self {
            instance,
            rpc_client,
        }
    }

    pub fn id(&self) -> &tendermint::node::Id {
        self.instance.peer_id()
    }
}
