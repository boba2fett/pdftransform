use axum::Router;
use axum::error_handling::HandleErrorLayer;
use service::state::ServiceCollection;
use service::routes;
use reqwest::StatusCode;
use tower_http::trace::TraceLayer;
use tracing::info;
use std::env;
use std::net::{SocketAddr, IpAddr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;
use tower::{timeout::TimeoutLayer, ServiceBuilder};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().json().finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not init tracing.");
    
    let mongo_uri = setup_mongo();
    let nats_uri = setup_nats();
    let expire_econds = setup_expire_time();

    let services = ServiceCollection::build(&mongo_uri, expire_econds, &nats_uri).await.unwrap();

    let app = Router::new()
        .merge(routes::root::create_route())
        .merge(routes::files::create_route(services.persistence.file_storage.clone()))
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
