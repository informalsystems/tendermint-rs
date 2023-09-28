use tendermint_rpc::{Client, HttpClient};

use std::env;

#[tokio::main]
async fn main() {
    let url = env::args()
        .skip(1)
        .next()
        .expect("should be run with an URL argument");
    let proxy_url = if let Ok(url) = env::var("https_proxy") {
        url
    } else if let Ok(url) = env::var("http_proxy") {
        url
    } else {
        panic!("no proxy URL configured");
    };
    let client = HttpClient::new_with_proxy(&*url, &*proxy_url).unwrap();

    let status = client.status().await.unwrap();

    println!("{status:?}");
}
