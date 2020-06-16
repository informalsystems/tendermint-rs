//! Tendermint RPC Proxy

#![allow(dead_code)]

use warp::{path, reject, Filter, Rejection, Reply};

use crate::rpc;

impl reject::Reject for rpc::Error {}

impl From<rpc::Error> for Rejection {
    fn from(err: rpc::Error) -> Self {
        reject::custom(err)
    }
}

struct Proxy {
    client: rpc::Client,
}

impl Proxy {
    fn new(client: rpc::Client) -> Self {
        Self { client }
    }

    async fn serve() {
        todo!()
    }
}

fn filters(client: rpc::Client) -> impl Filter<Extract = impl Reply, Error = Rejection> {
    health_filter(client)
}

fn health_filter(client: rpc::Client) -> impl Filter<Extract = impl Reply, Error = Rejection> {
    path("health")
        .and(warp::get())
        .and(path::end())
        .and(warp::any().map(move || client.clone()))
        .and_then(handler::health)
}

mod handler {
    use std::convert::Infallible;
    use warp::http::StatusCode;
    use warp::{reply, Rejection, Reply};

    use crate::rpc;

    pub async fn health(client: rpc::Client) -> Result<impl Reply, Rejection> {
        client.health().await?;

        Ok(reply())
    }

    pub async fn recover(err: Rejection) -> Result<impl Reply, Infallible> {
        // TODO(xla): Log and trace error.
        let (status, res) = {
            if err.is_not_found() {
                (StatusCode::NOT_FOUND, reply::json(&"Not found".to_string()))
            } else if let Some(rpc_err) = err.find::<rpc::Error>() {
                (StatusCode::INTERNAL_SERVER_ERROR, reply::json(&rpc_err))
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    reply::json(&"Something went wrong".to_string()),
                )
            }
        };

        Ok(reply::with_status(res, status))
    }

    #[cfg(test)]
    mod test {
        use futures::stream::TryStreamExt;
        use pretty_assertions::assert_eq;
        use serde_json::{json, Value};
        use warp::{reject, Rejection, Reply as _};

        use crate::rpc;

        #[tokio::test]
        async fn recover_custom() {
            let err = rpc::Error::invalid_params("key field must be alphanumeric");
            let have: Value = recover(reject::custom(err.clone())).await;
            let want = json!({
                "code": i32::from(err.code()),
                "data": err.data(),
                "message": err.message(),
            });

            assert_eq!(have, want);
        }

        #[tokio::test]
        async fn recover_not_found() {
            let have: Value = recover(reject::not_found()).await;
            let want = json!("Not found");

            assert_eq!(have, want);
        }

        async fn recover(err: Rejection) -> Value {
            let res = super::recover(err).await.unwrap();
            let body = res
                .into_response()
                .body_mut()
                .try_fold(Vec::new(), |mut data, chunk| async move {
                    data.extend_from_slice(&chunk);
                    Ok(data)
                })
                .await
                .unwrap();

            serde_json::from_slice(&body).unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use warp::http::StatusCode;
    use warp::test::request;
    use warp::Filter as _;

    use crate::rpc;

    #[tokio::test]
    async fn health() -> Result<(), rpc::Error> {
        let client = rpc::Client::new("tcp://127.0.0.1:0".parse().unwrap());
        let api = super::filters(client).recover(super::handler::recover);

        let res = request().method("GET").path("/health").reply(&api).await;

        assert_eq!(
            res.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "response status not {}, the body is:\n{:#?}",
            StatusCode::OK,
            res.body()
        );

        Ok(())
    }
}
