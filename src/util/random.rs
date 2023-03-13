use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub fn generate_30_alphanumeric() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect()
}
