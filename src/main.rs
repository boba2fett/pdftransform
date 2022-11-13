use std::env;
use pdftransform::persistence::DbClient;
use pdftransform::routes::{root, job, create_job, file};
use rocket::{launch, routes};
use rocket_db_pools::Database;

#[launch]
async fn rocket() -> _ {
    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    env::set_var("ROCKET_DATABASES", format!("{{db={{url=\"{mongo_uri}\"}}}}"));

    json_env_logger::init();
    rocket::build()
        .attach(DbClient::init())
        .mount("/", routes![root, job, create_job, file])
}
