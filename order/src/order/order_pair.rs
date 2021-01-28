extern crate uuid;
use uuid::Uuid;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Pair {
    pub price_ticker: String,
    pub ref_ticker: String,
    pub uuid: String
}

impl Pair {
    pub fn new(price_ticker: &str, ref_ticker: &str, uuid: Uuid) -> Self {
        Pair {
            price_ticker: String::from(price_ticker),
            ref_ticker: String::from(ref_ticker),
            uuid: uuid.to_string()
        }
    }
    pub fn serialize(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap();
        serialized
    }
    pub fn deserialize(json: &String) -> Self {
        let deserialized: Pair = serde_json::from_str(json).unwrap();
        deserialized
    }
}