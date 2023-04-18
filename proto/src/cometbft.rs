//! Auto-generated sub-modules for CometBFT

pub mod abci {
    pub mod v1 {
        include!("prost/cometbft.abci.v1.rs");
    }
    pub mod v2 {
        include!("prost/cometbft.abci.v2.rs");
    }
    pub mod v3 {
        include!("prost/cometbft.abci.v3.rs");
    }
}

pub mod blockchain {
    pub mod v1 {
        include!("prost/cometbft.blockchain.v1.rs");
    }
}

pub mod blocksync {
    pub mod v1 {
        include!("prost/cometbft.blocksync.v1.rs");
    }
    pub mod v2 {
        include!("prost/cometbft.blocksync.v2.rs");
    }
}

pub mod consensus {
    pub mod v1 {
        include!("prost/cometbft.consensus.v1.rs");
    }
    pub mod v2 {
        include!("prost/cometbft.consensus.v2.rs");
    }
}

pub mod crypto {
    pub mod v1 {
        include!("prost/cometbft.crypto.v1.rs");
    }
}

pub mod libs {
    pub mod bits {
        pub mod v1 {
            include!("prost/cometbft.libs.bits.v1.rs");
        }
    }
}

pub mod mempool {
    pub mod v1 {
        include!("prost/cometbft.mempool.v1.rs");
    }
}

pub mod p2p {
    pub mod v1 {
        include!("prost/cometbft.p2p.v1.rs");
    }
}

pub mod privval {
    pub mod v1 {
        include!("prost/cometbft.privval.v1.rs");
    }
    pub mod v2 {
        include!("prost/cometbft.privval.v2.rs");
    }
}

pub mod rpc {
    pub mod grpc {
        pub mod v1 {
            include!("prost/cometbft.rpc.grpc.v1.rs");
        }
        pub mod v2 {
            include!("prost/cometbft.rpc.grpc.v2.rs");
        }
        pub mod v3 {
            include!("prost/cometbft.rpc.grpc.v3.rs");
        }
    }
}

pub mod state {
    pub mod v1 {
        include!("prost/cometbft.state.v1.rs");
    }
    pub mod v2 {
        include!("prost/cometbft.state.v2.rs");
    }
    pub mod v3 {
        include!("prost/cometbft.state.v3.rs");
    }
}

pub mod statesync {
    pub mod v1 {
        include!("prost/cometbft.statesync.v1.rs");
    }
}

pub mod store {
    pub mod v1 {
        include!("prost/cometbft.store.v1.rs");
    }
}

pub mod types {
    pub mod v1 {
        include!("prost/cometbft.types.v1.rs");
    }
    pub mod v2 {
        include!("prost/cometbft.types.v2.rs");
    }
    pub mod v3 {
        include!("prost/cometbft.types.v3.rs");
    }
}

pub mod version {
    pub mod v1 {
        include!("prost/cometbft.version.v1.rs");
    }
}

pub mod meta {
    pub const REPOSITORY: &str = "https://github.com/cometbft/cometbft";
    pub const COMMITISH: &str = "mikhail/proto-version-suffixes";
}
