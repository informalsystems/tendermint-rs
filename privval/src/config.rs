//! Validator configuration.

use std::{net::SocketAddr, path::PathBuf};

use tonic::transport::ServerTlsConfig;

/// The common options for listening for gRPC connections.
#[derive(Debug, Clone)]
pub enum GrpcSocket {
    /// the syntax in Tendermint config is "grpc://HOST:PORT"
    Tcp(SocketAddr),
    /// the syntax in Tendermint config is "grpc://unix:PATH"
    Unix(PathBuf),
}

/// The basic configuration for the gRPC server.
#[derive(Debug, Clone)]
pub struct BasicServerConfig {
    /// The optional TLS configuration.
    pub tls_config: Option<ServerTlsConfig>,
    /// The choice of a socket type to listen on.
    pub socket: GrpcSocket,
}

impl BasicServerConfig {
    /// creates a basic config for the gRPC server
    pub fn new(tls_config: Option<ServerTlsConfig>, socket: GrpcSocket) -> Self {
        Self { tls_config, socket }
    }
}
