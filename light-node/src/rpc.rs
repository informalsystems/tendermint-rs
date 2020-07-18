//! JSONRPC Server and Client for the light-node RPC endpoint.
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, ServerBuilder};

use tendermint_light_client::supervisor::Handle;

use crate::error;

pub use sealed::{Client, Rpc, Server};

/// Run the given [`Server`] on the given address and blocks until closed.
///
/// n.b. The underlying server has semantics to close on drop. Also does it does not offer any way
/// to get the underlying Future to await, so we are left with this rather rudimentary way to
/// control the lifecycle. Should we be interested in a more controlled way to close the server we
/// can expose a handle in the future.
pub fn run<H>(server: Server<H>, addr: &str) -> Result<(), error::Error>
where
    H: Handle + Send + Sync + 'static,
{
    let mut io = IoHandler::new();
    io.extend_with(server.to_delegate());

    let srv = ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Any,
        ]))
        .start_http(&addr.parse().map_err(error::Kind::from)?)
        .map_err(|e| error::Kind::Io.context(e))?;

    srv.wait();

    Ok(())
}

mod sealed {
    use jsonrpc_core::futures::future::{self, Future, FutureResult};
    use jsonrpc_core::types::Error;
    use jsonrpc_derive::rpc;

    use tendermint_light_client::supervisor::Handle;
    use tendermint_light_client::types::LatestStatus;
    use tendermint_light_client::types::LightBlock;

    #[rpc]
    pub trait Rpc {
        /// Returns the latest trusted block.
        #[rpc(name = "state")]
        fn state(&self) -> FutureResult<Option<LightBlock>, Error>;

        /// Returns the latest status.
        #[rpc(name = "status")]
        fn status(&self) -> FutureResult<LatestStatus, Error>;
    }

    pub use self::rpc_impl_Rpc::gen_client::Client;

    pub struct Server<H>
    where
        H: Handle + Send + Sync,
    {
        handle: H,
    }

    impl<H> Server<H>
    where
        H: Handle + Send + Sync,
    {
        pub fn new(handle: H) -> Self {
            Self { handle }
        }
    }

    impl<H> Rpc for Server<H>
    where
        H: Handle + Send + Sync + 'static,
    {
        fn state(&self) -> FutureResult<Option<LightBlock>, Error> {
            let res = self.handle.latest_trusted().map_err(|e| {
                let mut err = Error::internal_error();
                err.message = e.to_string();
                err.data = serde_json::to_value(e.kind()).ok();
                err
            });

            future::result(res)
        }

        fn status(&self) -> FutureResult<LatestStatus, Error> {
            let res = self.handle.latest_status().map_err(|e| {
                let mut err = Error::internal_error();
                err.message = e.to_string();
                err.data = serde_json::to_value(e.kind()).ok();
                err
            });

            future::result(res)
        }
    }
}

#[cfg(test)]
mod test {
    use futures::compat::Future01CompatExt as _;
    use jsonrpc_core::futures::future::Future;
    use jsonrpc_core::IoHandler;
    use jsonrpc_core_client::transports::local;
    use pretty_assertions::assert_eq;

    use tendermint_light_client::errors::Error;
    use tendermint_light_client::supervisor::Handle;
    use tendermint_light_client::types::LatestStatus;
    use tendermint_light_client::types::LightBlock;

    use super::{Client, Rpc as _, Server};

    #[tokio::test]
    async fn state() {
        let server = Server::new(MockHandle {});
        let fut = {
            let mut io = IoHandler::new();
            io.extend_with(server.to_delegate());
            let (client, server) = local::connect::<Client, _, _>(io);
            client.state().join(server)
        };
        let (have, _) = fut.compat().await.unwrap();
        let want = serde_json::from_str(LIGHTBLOCK_JSON).unwrap();

        assert_eq!(have, want);
    }

    #[tokio::test]
    async fn status() {
        let server = Server::new(MockHandle {});
        let fut = {
            let mut io = IoHandler::new();
            io.extend_with(server.to_delegate());
            let (client, server) = local::connect::<Client, _, _>(io);
            client.status().join(server)
        };
        let (have, _) = fut.compat().await.unwrap();
        let want = serde_json::from_str(STATUS_JSON).unwrap();

        assert_eq!(have, want);
    }

    struct MockHandle;

    impl Handle for MockHandle {
        fn latest_trusted(&self) -> Result<Option<LightBlock>, Error> {
            let block: LightBlock = serde_json::from_str(LIGHTBLOCK_JSON).unwrap();

            Ok(Some(block))
        }
        fn latest_status(&self) -> Result<LatestStatus, Error> {
            let status: LatestStatus = serde_json::from_str(STATUS_JSON).unwrap();

            Ok(status)
        }
    }

    const LIGHTBLOCK_JSON: &str = r#"
{
    "signed_header": {
            "header": {
                    "version": {
                            "block": "0",
                            "app": "0"
                    },
                    "chain_id": "test-chain-01",
                    "height": "1",
                    "time": "2019-11-02T15:04:00Z",
                    "last_block_id": {
                            "hash": "",
                            "parts": {
                                    "total": "0",
                                    "hash": ""
                            }
                    },
                    "last_commit_hash": "",
                    "data_hash": "",
                    "validators_hash": "ADAE23D9D908638F3866C11A39E31CE4399AE6DE8EC8EBBCB1916B90C46EDDE3",
                    "next_validators_hash": "ADAE23D9D908638F3866C11A39E31CE4399AE6DE8EC8EBBCB1916B90C46EDDE3",
                    "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
                    "app_hash": "6170705F68617368",
                    "last_results_hash": "",
                    "evidence_hash": "",
                    "proposer_address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33"
            },
            "commit": {
                    "height": "1",
                    "round": "1",
                    "block_id": {
                            "hash": "76B0FB738138A2C934300D7B23C280B65965D7427DA4D5414B41C75EBC4AD4C3",
                            "parts": {
                                    "total": "1",
                                    "hash": "073CE26981DF93820595E602CE63B810BC8F1003D6BB28DEDFF5B2F4F09811A1"
                            }
                    },
                    "signatures": [
                            {
                                    "block_id_flag": 2,
                                    "validator_address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                                    "timestamp": "2019-11-02T15:04:10Z",
                                    "signature": "NaNXQhv7SgBtcq+iHwItxlYUMGHP5MeFpTbyNsnLtzwM6P/EAAAexUH94+osvRDoiahUOoQrRlTiZrYGfahWBw=="
                            },
                            {
                                    "block_id_flag": 2,
                                    "validator_address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                                    "timestamp": "2019-11-02T15:04:10Z",
                                    "signature": "tw0csJ1L1vkBG/71BMjrFEcA6VWjOx29WMwkg1cmDn82XBjRFz+HJu7amGoIj6WLL2p26pO25yQR49crsYQ+AA=="
                            }
                    ]
            }
    },
    "validator_set": {
            "validators": [
                    {
                            "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                            },
                            "voting_power": "50",
                            "proposer_priority": "-50"
                    },
                    {
                            "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                            },
                            "voting_power": "50",
                            "proposer_priority": "50"
                    }
            ],
            "proposer": {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                            "type": "tendermint/PubKeyEd25519",
                            "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "50",
                    "proposer_priority": "-50"
            }
    },
    "next_validator_set": {
            "validators": [
                    {
                            "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                            },
                            "voting_power": "50",
                            "proposer_priority": "0"
                    },
                    {
                            "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                            "pub_key": {
                                    "type": "tendermint/PubKeyEd25519",
                                    "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                            },
                            "voting_power": "50",
                            "proposer_priority": "0"
                    }
            ],
            "proposer": {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                            "type": "tendermint/PubKeyEd25519",
                            "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "50",
                    "proposer_priority": "0" }
    },
    "provider": "9D61B19DEFFD5A60BA844AF492EC2CC44449C569"
}
"#;
    const STATUS_JSON: &str = r#"
{
    "block_hash": "5A55D7AF2DF9AE4BF4B46FDABBBAD1B66D37B5E044A4843AB0FB0EBEC3E0422C",
    "connected_nodes": [
      "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE",
      "CEFEEDBADFADAD0C0CEEFACADE0ADEADBEEFC0FF"
    ],
    "height": 1565,
    "valset_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3"
}"#;
}
