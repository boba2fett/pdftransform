use std::{env, time::Duration};

use common::util::state::{NatsBaseSettings, S3BaseSettings};
use pdfium_render::prelude::Pdfium;
use preview::{preview::init_pdfium, state::ServiceCollection};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().json().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not init tracing.");
    
    let nats_uri = get_nats();
    let stream = get_stream();
    let consumer = get_consumer();
    let consumer_ack_wait = get_consumer_ack_wait();
    let bucket = get_bucket();
    let max_age = get_max_age();
    let max_deliver = get_max_deliver();
    let parallelism = get_parallelism();
    let pdfium = get_pdfium();

    let nats_settings = NatsBaseSettings {
        nats_uri: &nats_uri,
        bucket,
        max_age,
    };

    let s3_settings = get_s3_settings(max_age);

    let worker = ServiceCollection::build(nats_settings, stream, parallelism, pdfium, s3_settings, consumer, max_deliver, consumer_ack_wait).await.unwrap();
    worker.subscribe_service.subscribe().await.unwrap();
}

fn get_nats() -> String {
    env::var("NATS_URI").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

fn get_stream() -> String {
    env::var("NATS_JETSTREAM_QUEUE").unwrap_or_else(|_| "newJob".to_string())
}

fn get_consumer() -> String {
    env::var("NATS_JETSTREAM_CONSUMER").unwrap_or_else(|_| "preview".to_string())
}

fn get_bucket() -> String {
    env::var("NATS_KV_STORE_BUCKET").unwrap_or_else(|_| "job".to_string())
}

fn get_max_age() -> Duration {
    let max_age = env::var("MAX_AGE_SECONDS").map(|expire| expire.parse::<u64>());

    let max_age = match max_age {
        Ok(Ok(max_age)) => max_age,
        _ => 60 * 60 * 25,
    };
    Duration::from_secs(max_age)
}

fn get_max_deliver() -> i64 {
    let max_deliver = env::var("NATS_JETSTREAM_CONSUMER_MAX_DELIVERIES").map(|expire| expire.parse::<i64>());

    match max_deliver {
        Ok(Ok(max_deliver)) => max_deliver,
        _ => 5,
    }
}

fn get_consumer_ack_wait() -> Duration {
    let consumer_ack_wait = env::var("NATS_JETSTREAM_CONSUMER_ACK_WAIT_SECONDS").map(|expire| expire.parse::<u64>());

    let consumer_ack_wait = match consumer_ack_wait {
        Ok(Ok(consumer_ack_wait)) => consumer_ack_wait,
        _ => 30,
    };
    Duration::from_secs(consumer_ack_wait)
}

fn get_parallelism() -> usize {
    let parallelism = env::var("PARALLELISM").map(|expire| expire.parse::<usize>());
    match parallelism {
        Ok(Ok(parallelism)) if parallelism > 0 => parallelism,
        _ => 10,
    }
}

fn get_pdfium() -> Pdfium {
    init_pdfium().unwrap()
}

fn get_s3_settings(max_age: Duration) -> S3BaseSettings {
    S3BaseSettings {
        endpoint: env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
        region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        access_key_id: env::var("S3_ACCESS_KEY_ID").unwrap_or_else(|_| "access_key".to_string()),
        secret_access_key: env::var("S3_SECRET_ACCESS_KEY").unwrap_or_else(|_| "secret_key".to_string()),
        bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "bucket".to_string()),
        expire_seconds: max_age.as_secs() as u32,
    }
}