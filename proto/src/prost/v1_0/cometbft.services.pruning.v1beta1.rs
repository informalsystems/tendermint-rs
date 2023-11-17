#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetBlockRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetBlockRetainHeightResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockRetainHeightRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockRetainHeightResponse {
    /// The retain height set by the application.
    #[prost(uint64, tag = "1")]
    pub app_retain_height: u64,
    /// The retain height set via the pruning service (e.g. by the data
    /// companion) specifically for blocks.
    #[prost(uint64, tag = "2")]
    pub pruning_service_retain_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetBlockResultsRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetBlockResultsRetainHeightResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockResultsRetainHeightRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockResultsRetainHeightResponse {
    /// The retain height set by the pruning service (e.g. by the data
    /// companion) specifically for block results.
    #[prost(uint64, tag = "1")]
    pub pruning_service_retain_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetTxIndexerRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetTxIndexerRetainHeightResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxIndexerRetainHeightRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxIndexerRetainHeightResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetBlockIndexerRetainHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetBlockIndexerRetainHeightResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockIndexerRetainHeightRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockIndexerRetainHeightResponse {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
/// Generated server implementations.
pub mod pruning_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with PruningServiceServer.
    #[async_trait]
    pub trait PruningService: Send + Sync + 'static {
        /// SetBlockRetainHeightRequest indicates to the node that it can safely
        /// prune all block data up to the specified retain height.
        ///
        /// The lower of this retain height and that set by the application in its
        /// Commit response will be used by the node to determine which heights' data
        /// can be pruned.
        async fn set_block_retain_height(
            &self,
            request: tonic::Request<super::SetBlockRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetBlockRetainHeightResponse>,
            tonic::Status,
        >;
        /// GetBlockRetainHeight returns information about the retain height
        /// parameters used by the node to influence block retention/pruning.
        async fn get_block_retain_height(
            &self,
            request: tonic::Request<super::GetBlockRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockRetainHeightResponse>,
            tonic::Status,
        >;
        /// SetBlockResultsRetainHeightRequest indicates to the node that it can
        /// safely prune all block results data up to the specified height.
        ///
        /// The node will always store the block results for the latest height to
        /// help facilitate crash recovery.
        async fn set_block_results_retain_height(
            &self,
            request: tonic::Request<super::SetBlockResultsRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetBlockResultsRetainHeightResponse>,
            tonic::Status,
        >;
        /// GetBlockResultsRetainHeight returns information about the retain height
        /// parameters used by the node to influence block results retention/pruning.
        async fn get_block_results_retain_height(
            &self,
            request: tonic::Request<super::GetBlockResultsRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockResultsRetainHeightResponse>,
            tonic::Status,
        >;
        /// SetTxIndexerRetainHeightRequest indicates to the node that it can safely
        /// prune all tx indices up to the specified retain height.
        async fn set_tx_indexer_retain_height(
            &self,
            request: tonic::Request<super::SetTxIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetTxIndexerRetainHeightResponse>,
            tonic::Status,
        >;
        /// GetTxIndexerRetainHeight returns information about the retain height
        /// parameters used by the node to influence TxIndexer pruning
        async fn get_tx_indexer_retain_height(
            &self,
            request: tonic::Request<super::GetTxIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTxIndexerRetainHeightResponse>,
            tonic::Status,
        >;
        /// SetBlockIndexerRetainHeightRequest indicates to the node that it can safely
        /// prune all block indices up to the specified retain height.
        async fn set_block_indexer_retain_height(
            &self,
            request: tonic::Request<super::SetBlockIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetBlockIndexerRetainHeightResponse>,
            tonic::Status,
        >;
        /// GetBlockIndexerRetainHeight returns information about the retain height
        /// parameters used by the node to influence BlockIndexer pruning
        async fn get_block_indexer_retain_height(
            &self,
            request: tonic::Request<super::GetBlockIndexerRetainHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockIndexerRetainHeightResponse>,
            tonic::Status,
        >;
    }
    /// PruningService provides privileged access to specialized pruning
    /// functionality on the CometBFT node to help control node storage.
    #[derive(Debug)]
    pub struct PruningServiceServer<T: PruningService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: PruningService> PruningServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PruningServiceServer<T>
    where
        T: PruningService,
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
                "/cometbft.services.pruning.v1beta1.PruningService/SetBlockRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct SetBlockRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<super::SetBlockRetainHeightRequest>
                    for SetBlockRetainHeightSvc<T> {
                        type Response = super::SetBlockRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetBlockRetainHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::set_block_retain_height(
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
                        let method = SetBlockRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/GetBlockRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<super::GetBlockRetainHeightRequest>
                    for GetBlockRetainHeightSvc<T> {
                        type Response = super::GetBlockRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockRetainHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::get_block_retain_height(
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
                        let method = GetBlockRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/SetBlockResultsRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct SetBlockResultsRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<
                        super::SetBlockResultsRetainHeightRequest,
                    > for SetBlockResultsRetainHeightSvc<T> {
                        type Response = super::SetBlockResultsRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::SetBlockResultsRetainHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::set_block_results_retain_height(
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
                        let method = SetBlockResultsRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/GetBlockResultsRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockResultsRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<
                        super::GetBlockResultsRetainHeightRequest,
                    > for GetBlockResultsRetainHeightSvc<T> {
                        type Response = super::GetBlockResultsRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetBlockResultsRetainHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::get_block_results_retain_height(
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
                        let method = GetBlockResultsRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/SetTxIndexerRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct SetTxIndexerRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<super::SetTxIndexerRetainHeightRequest>
                    for SetTxIndexerRetainHeightSvc<T> {
                        type Response = super::SetTxIndexerRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::SetTxIndexerRetainHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::set_tx_indexer_retain_height(
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
                        let method = SetTxIndexerRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/GetTxIndexerRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxIndexerRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<super::GetTxIndexerRetainHeightRequest>
                    for GetTxIndexerRetainHeightSvc<T> {
                        type Response = super::GetTxIndexerRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetTxIndexerRetainHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::get_tx_indexer_retain_height(
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
                        let method = GetTxIndexerRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/SetBlockIndexerRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct SetBlockIndexerRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<
                        super::SetBlockIndexerRetainHeightRequest,
                    > for SetBlockIndexerRetainHeightSvc<T> {
                        type Response = super::SetBlockIndexerRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::SetBlockIndexerRetainHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::set_block_indexer_retain_height(
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
                        let method = SetBlockIndexerRetainHeightSvc(inner);
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
                "/cometbft.services.pruning.v1beta1.PruningService/GetBlockIndexerRetainHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockIndexerRetainHeightSvc<T: PruningService>(pub Arc<T>);
                    impl<
                        T: PruningService,
                    > tonic::server::UnaryService<
                        super::GetBlockIndexerRetainHeightRequest,
                    > for GetBlockIndexerRetainHeightSvc<T> {
                        type Response = super::GetBlockIndexerRetainHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetBlockIndexerRetainHeightRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PruningService>::get_block_indexer_retain_height(
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
                        let method = GetBlockIndexerRetainHeightSvc(inner);
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
    impl<T: PruningService> Clone for PruningServiceServer<T> {
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
    impl<T: PruningService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PruningService> tonic::server::NamedService for PruningServiceServer<T> {
        const NAME: &'static str = "cometbft.services.pruning.v1beta1.PruningService";
    }
}
