use pdftransform::consts::{MAX_KIBIBYTES, PARALLELISM, PDFIUM};
use pdftransform::files;
use pdftransform::persistence::{self, DbClient};
use pdftransform::routes::*;
use pdftransform::transform::init_pdfium;
use rocket::{launch, routes};
use rocket_db_pools::Database;
use std::env;

#[launch]
async fn rocket() -> _ {
    json_env_logger2::init();
    setup_expire_time().await;
    setup_parallelism();
    setup_max_size();
    setup_pdfium();

    rocket::build()
        .attach(DbClient::init())
        .mount("/", routes![root_links, file, preview_sync, preview_job, create_preview_job, transform_job, create_transform_job, health,])
}

async fn setup_expire_time() {
    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    env::set_var("ROCKET_DATABASES", format!("{{db={{url=\"{mongo_uri}\"}}}}"));

    let expire = env::var("EXPIRE_AFTER_SECONDS").map(|expire| expire.parse::<u64>());

    let expire = match expire {
        Ok(Ok(expire)) if expire > 0 => expire,
        _ => 60 * 60 * 25,
    };
    let client = persistence::set_expire_after(&mongo_uri, expire).await.unwrap();

    files::set_expire_after(&client, expire).await.unwrap();
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
