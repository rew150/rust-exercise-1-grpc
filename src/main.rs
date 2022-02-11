use std::env;

use dotenv::dotenv;
use tonic::transport::Server;

mod proto;
mod service;

static DEFAULT_ADDRESS: &str = "127.0.0.1:50051";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().unwrap();
    let address = match env::var("ADDRESS") {
        Ok(a) => {
            println!("ADDRESS loaded from env = {}", &a);
            a
        },
        Err(_) => {
            println!("ADDRESS not found, use {}", DEFAULT_ADDRESS);
            DEFAULT_ADDRESS.to_owned()
        }
    }.parse().expect("parse address error");

    Server::builder()
        .add_service(service::datamap_server())
        .serve(address)
        .await.unwrap();
}
