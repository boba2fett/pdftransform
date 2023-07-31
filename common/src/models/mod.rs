mod preview;
pub use preview::*;

mod transform;
pub use transform::*;

mod jobs;
pub use jobs::*;

mod nats;
pub use nats::*;

pub trait ToIdJson: Send + Sync {
    fn to_json(&self) -> Result<String, &'static str>;
    fn get_id(&self) -> &str;
}
