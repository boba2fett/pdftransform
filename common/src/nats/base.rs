use async_nats::{connect, jetstream::Context};

pub struct BaseJetstream  {
    pub jetstream: Context,
}

impl BaseJetstream {
    pub async fn build(uri: &str) -> Result<Self, &'static str> {
        let nc = connect(uri).await.map_err(|_| "could not connect")?;
        let jetstream = async_nats::jetstream::new(nc);
        Ok(BaseJetstream {
            jetstream,
        })
    }
}