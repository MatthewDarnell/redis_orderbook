use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum OrderType {
    BID,
    ASK,
    DELETE
}
