use std::env;
use pdftransform::consts::PARALLELISM;
use pdftransform::persistence::{DbClient, self};
use pdftransform::files;
use pdftransform::routes::{root, job, create_job, file};
use rocket::{launch, routes};
use rocket_db_pools::Database;

#[launch]
async fn rocket() -> _ {
    setup_expire_time().await;
    setup_parallelism();

    json_env_logger2::init();
    rocket::build()
        .attach(DbClient::init())
        .mount("/", routes![root, job, create_job, file])
}

async fn setup_expire_time() {
    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    env::set_var("ROCKET_DATABASES", format!("{{db={{url=\"{mongo_uri}\"}}}}"));

    let expire = env::var("EXPIRE_AFTER_SECONDS").map(|expire| expire.parse::<u64>());

    let expire = match expire {
        Ok(Ok(expire)) if expire > 0 => expire,
        _ => 60*60*25,
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