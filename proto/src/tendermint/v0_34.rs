//! Tendermint-proto auto-generated sub-modules for Tendermint

pub mod abci {
    include!("../prost/v0_34/tendermint.abci.rs");
}

pub mod blockchain {
    include!("../prost/v0_34/tendermint.blockchain.rs");
}

pub mod consensus {
    include!("../prost/v0_34/tendermint.consensus.rs");
}

pub mod crypto {
    include!("../prost/v0_34/tendermint.crypto.rs");
}

pub mod libs {
    pub mod bits {
        include!("../prost/v0_34/tendermint.libs.bits.rs");
    }
}

pub mod mempool {
    include!("../prost/v0_34/tendermint.mempool.rs");
}

pub mod p2p {
    include!("../prost/v0_34/tendermint.p2p.rs");
}

pub mod privval {
    include!("../prost/v0_34/tendermint.privval.rs");
}

pub mod rpc {
    pub mod grpc {
        include!("../prost/v0_34/tendermint.rpc.grpc.rs");
    }
}

pub mod state {
    include!("../prost/v0_34/tendermint.state.rs");
}

pub mod statesync {
    include!("../prost/v0_34/tendermint.statesync.rs");
}

pub mod store {
    include!("../prost/v0_34/tendermint.store.rs");
}

pub mod types {
    include!("../prost/v0_34/tendermint.types.rs");
}

pub mod version {
    include!("../prost/v0_34/tendermint.version.rs");
}

pub mod meta {
    pub const REPOSITORY: &str = "https://github.com/cometbft/cometbft";
    pub const COMMITISH: &str = "v0.34.28";
}
