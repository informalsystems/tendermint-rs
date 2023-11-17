#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockResultsRequest {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestBlockResultsRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockResultsResponse {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(message, repeated, tag = "2")]
    pub tx_results: ::prost::alloc::vec::Vec<
        super::super::super::abci::v1beta3::ExecTxResult,
    >,
    #[prost(message, repeated, tag = "3")]
    pub finalize_block_events: ::prost::alloc::vec::Vec<
        super::super::super::abci::v1beta2::Event,
    >,
    #[prost(message, repeated, tag = "4")]
    pub validator_updates: ::prost::alloc::vec::Vec<
        super::super::super::abci::v1beta1::ValidatorUpdate,
    >,
    #[prost(message, optional, tag = "5")]
    pub consensus_param_updates: ::core::option::Option<
        super::super::super::types::v1beta3::ConsensusParams,
    >,
    #[prost(bytes = "vec", tag = "6")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestBlockResultsResponse {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(message, repeated, tag = "2")]
    pub tx_results: ::prost::alloc::vec::Vec<
        super::super::super::abci::v1beta3::ExecTxResult,
    >,
    #[prost(message, repeated, tag = "3")]
    pub finalize_block_events: ::prost::alloc::vec::Vec<
        super::super::super::abci::v1beta2::Event,
    >,
    #[prost(message, repeated, tag = "4")]
    pub validator_updates: ::prost::alloc::vec::Vec<
        super::super::super::abci::v1beta1::ValidatorUpdate,
    >,
    #[prost(message, optional, tag = "5")]
    pub consensus_param_updates: ::core::option::Option<
        super::super::super::types::v1beta3::ConsensusParams,
    >,
    #[prost(bytes = "vec", tag = "6")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
/// Generated server implementations.
#[cfg(feature = "grpc-server")]
pub mod block_results_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with BlockResultsServiceServer.
    #[async_trait]
    pub trait BlockResultsService: Send + Sync + 'static {
        /// GetBlockResults returns the BlockResults of the requested height.
        async fn get_block_results(
            &self,
            request: tonic::Request<super::GetBlockResultsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockResultsResponse>,
            tonic::Status,
        >;
        /// GetLatestBlockResults returns the BlockResults of the latest committed height.
        async fn get_latest_block_results(
            &self,
            request: tonic::Request<super::GetLatestBlockResultsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetLatestBlockResultsResponse>,
            tonic::Status,
        >;
    }
    ///
    /// BlockResultService provides the block results of a given or latestheight.
    #[derive(Debug)]
    pub struct BlockResultsServiceServer<T: BlockResultsService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: BlockResultsService> BlockResultsServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BlockResultsServiceServer<T>
    where
        T: BlockResultsService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cometbft.services.block_results.v1beta1.BlockResultsService/GetBlockResults" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockResultsSvc<T: BlockResultsService>(pub Arc<T>);
                    impl<
                        T: BlockResultsService,
                    > tonic::server::UnaryService<super::GetBlockResultsRequest>
                    for GetBlockResultsSvc<T> {
                        type Response = super::GetBlockResultsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockResultsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as BlockResultsService>::get_block_results(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockResultsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cometbft.services.block_results.v1beta1.BlockResultsService/GetLatestBlockResults" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestBlockResultsSvc<T: BlockResultsService>(pub Arc<T>);
                    impl<
                        T: BlockResultsService,
                    > tonic::server::UnaryService<super::GetLatestBlockResultsRequest>
                    for GetLatestBlockResultsSvc<T> {
                        type Response = super::GetLatestBlockResultsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetLatestBlockResultsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as BlockResultsService>::get_latest_block_results(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetLatestBlockResultsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: BlockResultsService> Clone for BlockResultsServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: BlockResultsService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BlockResultsService> tonic::server::NamedService
    for BlockResultsServiceServer<T> {
        const NAME: &'static str = "cometbft.services.block_results.v1beta1.BlockResultsService";
    }
}
