use axum::Router;
use kv_log_macro::info;
use pdftransform::consts::{MAX_KIBIBYTES, PARALLELISM, PDFIUM, MONGO_CLIENT};
use pdftransform::{persistence, files};
use pdftransform::routes;
use pdftransform::transform::init_pdfium;
use std::env;
use std::net::{SocketAddr, IpAddr, Ipv6Addr};

#[tokio::main]
async fn main() {
    json_env_logger2::init();
    setup_expire_time().await;
    setup_parallelism();
    setup_max_size();
    setup_pdfium();

    let app = Router::new()
        .merge(routes::root::create_route())
        .merge(routes::files::create_route())
        .merge(routes::preview::create_route())
        .merge(routes::transform::create_route());

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
        Ok(Ok(expire)) if expire > 0 => expire,
        _ => 60 * 60 * 25,
    };

    persistence::set_expire_after(expire).await.unwrap();
    files::set_expire_after(expire).await.unwrap();
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

fn setup_max_size() {
    let max_kibibytes = env::var("MAX_KIBIBYTES").map(|max| max.parse::<usize>());
    unsafe {
        MAX_KIBIBYTES = match max_kibibytes {
            Ok(Ok(max)) if max > 0 => max,
            _ => MAX_KIBIBYTES,
        }
    }
}

fn setup_pdfium() {
    let pdfium = init_pdfium();
    unsafe {
        PDFIUM = Some(pdfium);
    }
}
