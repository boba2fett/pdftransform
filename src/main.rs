use axum::Router;
use axum::error_handling::HandleErrorLayer;
use pdftransform::util::consts::{PARALLELISM, PDFIUM, MONGO_CLIENT};
use pdftransform::{persistence, files};
use pdftransform::routes;
use pdftransform::convert::transform::{init_pdfium, check_libre};
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
    setup_expire_time().await;
    setup_parallelism();
    setup_pdfium();
    setup_libre();

    let app = Router::new()
        .merge(routes::root::create_route())
        .merge(routes::files::create_route())
        .merge(routes::preview::create_route())
        .merge(routes::transform::create_route())
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

async fn setup_expire_time() {
    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let client = persistence::init_mongo(&mongo_uri).await.unwrap();
    unsafe {
        MONGO_CLIENT = Some(client)
    }

    let expire = env::var("EXPIRE_AFTER_SECONDS").map(|expire| expire.parse::<u64>());

    let expire = match expire {
        Ok(Ok(expire)) => expire,
        _ => 60 * 60 * 25,
    };
    if expire != 0 {
        persistence::set_expire_after(expire).await.unwrap();
        files::set_expire_after(expire).await.unwrap();
    }
}

fn setup_parallelism() {
    let parallelism = env::var("PARALLELISM").map(|expire| expire.parse::<usize>());
    unsafe {
        PARALLELISM = match parallelism {
            Ok(Ok(parallelism)) if parallelism > 0 => parallelism,
            _ => PARALLELISM,
        }
    }
}

fn setup_pdfium() {
    let pdfium = init_pdfium();
    unsafe {
        PDFIUM = Some(pdfium);
    }
}

fn setup_libre() {
    if !check_libre() {
        panic!("Libre not installed.")
    }
}
