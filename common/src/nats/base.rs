use async_nats::{connect, jetstream::Context};

pub struct BaseJetStream  {
    pub jetstream: Context,
}

impl BaseJetStream {
    pub async fn build(uri: &str) -> Result<Self, &'static str> {
        let nc = connect(uri).await.map_err(|_| "could not connect to nats")?;
        let jetstream = async_nats::jetstream::new(nc);
        Ok(BaseJetStream {
            jetstream,
        })
    }
}