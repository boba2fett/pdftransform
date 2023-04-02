use std::env;

use pdfium_render::prelude::Pdfium;
use preview::{preview::init_pdfium, state::Services};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().json().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not init tracing.");
    
    let mongo_uri = setup_mongo();
    let nats_uri = setup_nats();
    let expire_econds = setup_expire_time();
    let parallelism = setup_parallelism();
    let pdfium = setup_pdfium();

    let worker = Services::build(&mongo_uri, expire_econds, parallelism, &nats_uri, pdfium).await.unwrap();
    worker.subscriber.subscribe().await.unwrap();
}

fn setup_mongo() -> String {
    env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}

fn setup_nats() -> String {
    env::var("NATS_URI").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

fn setup_expire_time() -> u64 {
    let expire = env::var("EXPIRE_AFTER_SECONDS").map(|expire| expire.parse::<u64>());

    let expire = match expire {
        Ok(Ok(expire)) => expire,
        _ => 60 * 60 * 25,
    };
    expire
}

fn setup_parallelism() -> usize {
    let parallelism = env::var("PARALLELISM").map(|expire| expire.parse::<usize>());
    match parallelism {
        Ok(Ok(parallelism)) if parallelism > 0 => parallelism,
        _ => 10,
    }
}

fn setup_pdfium() -> Pdfium {
    init_pdfium().unwrap()
}
