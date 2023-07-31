use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageDLQModel {
    pub stream_seq: u64,
}
