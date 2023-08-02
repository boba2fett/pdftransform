use axum::Router;
use axum::error_handling::HandleErrorLayer;
use common::util::state::NatsBaseSettings;
use service::state::ServiceCollection;
use service::routes;
use reqwest::StatusCode;
use tower_http::trace::TraceLayer;
use tracing::info;
use std::env;
use std::net::{SocketAddr, IpAddr, Ipv6Addr};
use std::time::Duration;
use tower::{timeout::TimeoutLayer, ServiceBuilder};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().json().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not init tracing.");
    
    let nats_uri = get_nats();
    let stream = get_stream();
    let bucket = get_bucket();
    let max_age = get_max_age();

    let settings = NatsBaseSettings {
        nats_uri: &nats_uri,
        bucket,
        max_age,
    };

    let services = ServiceCollection::build(settings, stream).await.unwrap();

    let app = Router::new()
        .merge(routes::root::create_route())
        .merge(routes::preview::create_route(services.clone()))
        .merge(routes::transform::create_route(services.clone()))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(HandleErrorLayer::new(|_| async {
                StatusCode::REQUEST_TIMEOUT
            }))
            .layer(TimeoutLayer::new(Duration::from_secs(59))),
        );

    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 8000);
    info!("listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn get_nats() -> String {
    env::var("NATS_URI").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

fn get_stream() -> String {
    env::var("NATS_JETSTREAM_QUEUE").unwrap_or_else(|_| "newJob".to_string())
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
