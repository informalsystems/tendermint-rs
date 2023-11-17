#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetByHeightRequest {
    /// The height of the block requested. If set to 0, the latest height will be returned.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<super::super::super::types::v1beta1::BlockId>,
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<super::super::super::types::v1beta3::Block>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestResponse {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<super::super::super::types::v1beta1::BlockId>,
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<super::super::super::types::v1beta3::Block>,
}
/// GetLatestHeightRequest - empty message since no parameter is required
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestHeightRequest {}
/// GetLatestHeightResponse provides the height of the latest committed block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetLatestHeightResponse {
    /// The height of the latest committed block. Will be 0 if no data has been
    /// committed yet.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
/// Generated server implementations.
pub mod block_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with BlockServiceServer.
    #[async_trait]
    pub trait BlockService: Send + Sync + 'static {
        /// GetBlock retrieves the block information at a particular height.
        async fn get_by_height(
            &self,
            request: tonic::Request<super::GetByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetByHeightResponse>,
            tonic::Status,
        >;
        /// GetLatest retrieves the latest block.
        async fn get_latest(
            &self,
            request: tonic::Request<super::GetLatestRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetLatestResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the GetLatestHeight method.
        type GetLatestHeightStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::GetLatestHeightResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// GetLatestHeight returns a stream of the latest block heights committed by
        /// the network. This is a long-lived stream that is only terminated by the
        /// server if an error occurs. The caller is expected to handle such
        /// disconnections and automatically reconnect.
        async fn get_latest_height(
            &self,
            request: tonic::Request<super::GetLatestHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::GetLatestHeightStream>,
            tonic::Status,
        >;
    }
    /// BlockService provides information about blocks
    #[derive(Debug)]
    pub struct BlockServiceServer<T: BlockService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: BlockService> BlockServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for BlockServiceServer<T>
    where
        T: BlockService,
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
                "/cometbft.services.block.v1beta1.BlockService/GetByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetByHeightSvc<T: BlockService>(pub Arc<T>);
                    impl<
                        T: BlockService,
                    > tonic::server::UnaryService<super::GetByHeightRequest>
                    for GetByHeightSvc<T> {
                        type Response = super::GetByHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetByHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as BlockService>::get_by_height(&inner, request).await
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
                        let method = GetByHeightSvc(inner);
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
                "/cometbft.services.block.v1beta1.BlockService/GetLatest" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestSvc<T: BlockService>(pub Arc<T>);
                    impl<
                        T: BlockService,
                    > tonic::server::UnaryService<super::GetLatestRequest>
                    for GetLatestSvc<T> {
                        type Response = super::GetLatestResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetLatestRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as BlockService>::get_latest(&inner, request).await
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
                        let method = GetLatestSvc(inner);
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
                "/cometbft.services.block.v1beta1.BlockService/GetLatestHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetLatestHeightSvc<T: BlockService>(pub Arc<T>);
                    impl<
                        T: BlockService,
                    > tonic::server::ServerStreamingService<
                        super::GetLatestHeightRequest,
                    > for GetLatestHeightSvc<T> {
                        type Response = super::GetLatestHeightResponse;
                        type ResponseStream = T::GetLatestHeightStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetLatestHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as BlockService>::get_latest_height(&inner, request)
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
                        let method = GetLatestHeightSvc(inner);
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
                        let res = grpc.server_streaming(method, req).await;
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
    impl<T: BlockService> Clone for BlockServiceServer<T> {
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
    impl<T: BlockService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BlockService> tonic::server::NamedService for BlockServiceServer<T> {
        const NAME: &'static str = "cometbft.services.block.v1beta1.BlockService";
    }
}
