use std::{env, sync::Arc, collections::HashMap};

use dotenv::dotenv;
use tonic::transport::Server;
use tokio::{signal, sync::RwLock};

use crate::jsonfile::write_json;

mod proto;
mod service;
mod jsonfile;

static DEFAULT_ADDRESS: &str = "127.0.0.1:50051";
static DEFAULT_JSON_PATH: &str = "./data/data.json";

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

    let json_path = match env::var("JSON_PATH") {
        Ok(p) => {
            println!("JSON_PATH loaded from env = {}", &p);
            p
        },
        Err(_) => {
            println!("JSON_PATH not found, use {}", DEFAULT_JSON_PATH);
            DEFAULT_JSON_PATH.to_owned()
        }
    };

    // load data from json file
    let map = Arc::new(tokio::sync::RwLock::new(
        jsonfile::try_read_json_init(&json_path)
    ));

    Server::builder()
        .add_service(service::datamap_server(json_path.clone(), Arc::clone(&map)))
        .serve_with_shutdown(address, wait_for_ctrl_c(&json_path, &map))
        .await.unwrap();
    
    println!("Shutting down gracefully...");
}

async fn wait_for_ctrl_c(file_loc: &str, map: &RwLock<HashMap<String, i64>>) {
    signal::ctrl_c().await.expect("error waiting for signal");
    println!("saving file");
    write_json(file_loc, map).await;
}
