use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum OrderExecutionType {
    MARKET,
    LIMIT
}