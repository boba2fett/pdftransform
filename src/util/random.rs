use rand::{thread_rng, distributions::Alphanumeric, Rng};

pub fn generate_30_alphanumeric() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect()
}