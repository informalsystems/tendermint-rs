#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoteSignerError {
    #[prost(int32, tag="1")]
    pub code: i32,
    #[prost(string, tag="2")]
    pub description: ::prost::alloc::string::String,
}
/// PubKeyRequest requests the consensus public key from the remote signer.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyRequest {
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
}
/// PubKeyResponse is a response message containing the public key.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKeyResponse {
    #[prost(message, optional, tag="1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(message, optional, tag="2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
/// SignVoteRequest is a request to sign a vote
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignVoteRequest {
    #[prost(message, optional, tag="1")]
    pub vote: ::core::option::Option<super::types::Vote>,
    #[prost(string, tag="2")]
    pub chain_id: ::prost::alloc::string::String,
}
/// SignedVoteResponse is a response containing a signed vote or an error
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedVoteResponse {
    #[prost(message, optional, tag="1")]
    pub vote: ::core::option::Option<super::types::Vote>,
    #[prost(message, optional, tag="2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
/// SignProposalRequest is a request to sign a proposal
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignProposalRequest {
    #[prost(message, optional, tag="1")]
    pub proposal: ::core::option::Option<super::types::Proposal>,
    #[prost(string, tag="2")]
    pub chain_id: ::prost::alloc::string::String,
}
/// SignedProposalResponse is response containing a signed proposal or an error
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedProposalResponse {
    #[prost(message, optional, tag="1")]
    pub proposal: ::core::option::Option<super::types::Proposal>,
    #[prost(message, optional, tag="2")]
    pub error: ::core::option::Option<RemoteSignerError>,
}
/// PingRequest is a request to confirm that the connection is alive.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingRequest {
}
/// PingResponse is a response to confirm that the connection is alive.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(oneof="message::Sum", tags="1, 2, 3, 4, 5, 6, 7, 8")]
    pub sum: ::core::option::Option<message::Sum>,
}
/// Nested message and enum types in `Message`.
pub mod message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag="1")]
        PubKeyRequest(super::PubKeyRequest),
        #[prost(message, tag="2")]
        PubKeyResponse(super::PubKeyResponse),
        #[prost(message, tag="3")]
        SignVoteRequest(super::SignVoteRequest),
        #[prost(message, tag="4")]
        SignedVoteResponse(super::SignedVoteResponse),
        #[prost(message, tag="5")]
        SignProposalRequest(super::SignProposalRequest),
        #[prost(message, tag="6")]
        SignedProposalResponse(super::SignedProposalResponse),
        #[prost(message, tag="7")]
        PingRequest(super::PingRequest),
        #[prost(message, tag="8")]
        PingResponse(super::PingResponse),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Errors {
    Unknown = 0,
    UnexpectedResponse = 1,
    NoConnection = 2,
    ConnectionTimeout = 3,
    ReadTimeout = 4,
    WriteTimeout = 5,
}
impl Errors {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Errors::Unknown => "ERRORS_UNKNOWN",
            Errors::UnexpectedResponse => "ERRORS_UNEXPECTED_RESPONSE",
            Errors::NoConnection => "ERRORS_NO_CONNECTION",
            Errors::ConnectionTimeout => "ERRORS_CONNECTION_TIMEOUT",
            Errors::ReadTimeout => "ERRORS_READ_TIMEOUT",
            Errors::WriteTimeout => "ERRORS_WRITE_TIMEOUT",
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "grpc")]
pub mod priv_validator_api_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with PrivValidatorApiServer.
    #[async_trait]
    pub trait PrivValidatorApi: Send + Sync + 'static {
        async fn get_pub_key(
            &self,
            request: tonic::Request<super::PubKeyRequest>,
        ) -> Result<tonic::Response<super::PubKeyResponse>, tonic::Status>;
        async fn sign_vote(
            &self,
            request: tonic::Request<super::SignVoteRequest>,
        ) -> Result<tonic::Response<super::SignedVoteResponse>, tonic::Status>;
        async fn sign_proposal(
            &self,
            request: tonic::Request<super::SignProposalRequest>,
        ) -> Result<tonic::Response<super::SignedProposalResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct PrivValidatorApiServer<T: PrivValidatorApi> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: PrivValidatorApi> PrivValidatorApiServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
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
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PrivValidatorApiServer<T>
    where
        T: PrivValidatorApi,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/tendermint.privval.PrivValidatorAPI/GetPubKey" => {
                    #[allow(non_camel_case_types)]
                    struct GetPubKeySvc<T: PrivValidatorApi>(pub Arc<T>);
                    impl<
                        T: PrivValidatorApi,
                    > tonic::server::UnaryService<super::PubKeyRequest>
                    for GetPubKeySvc<T> {
                        type Response = super::PubKeyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PubKeyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_pub_key(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPubKeySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.privval.PrivValidatorAPI/SignVote" => {
                    #[allow(non_camel_case_types)]
                    struct SignVoteSvc<T: PrivValidatorApi>(pub Arc<T>);
                    impl<
                        T: PrivValidatorApi,
                    > tonic::server::UnaryService<super::SignVoteRequest>
                    for SignVoteSvc<T> {
                        type Response = super::SignedVoteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SignVoteRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).sign_vote(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SignVoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/tendermint.privval.PrivValidatorAPI/SignProposal" => {
                    #[allow(non_camel_case_types)]
                    struct SignProposalSvc<T: PrivValidatorApi>(pub Arc<T>);
                    impl<
                        T: PrivValidatorApi,
                    > tonic::server::UnaryService<super::SignProposalRequest>
                    for SignProposalSvc<T> {
                        type Response = super::SignedProposalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SignProposalRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).sign_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SignProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
    impl<T: PrivValidatorApi> Clone for PrivValidatorApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: PrivValidatorApi> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PrivValidatorApi> tonic::transport::NamedService
    for PrivValidatorApiServer<T> {
        const NAME: &'static str = "tendermint.privval.PrivValidatorAPI";
    }
}
