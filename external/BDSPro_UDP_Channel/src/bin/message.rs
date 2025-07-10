use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub typ: String,            // "DATA", "RQST"
    pub seq: u64,               // Sequenznummer
    pub data: Option<String>,   // Nutzdaten (optional)
    
}

fn main() {
    println!("This binary is deprecated. Use the library instead.");
}