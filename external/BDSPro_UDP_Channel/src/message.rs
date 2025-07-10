use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub typ: String,            // "DATA", "RQST", "ACK"
    pub seq: u64,               // Sequence number
    pub data: Option<String>,   // Payload data (optional)
}