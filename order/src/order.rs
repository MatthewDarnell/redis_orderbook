extern crate uuid;
use uuid::Uuid;
use std::time::SystemTime;
pub mod order_type;
pub mod order_execution_type;
pub mod order_executor;
pub mod order_executor_result;
pub mod orderbook;
pub mod store;
pub mod order_pair;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub uuid: Uuid,
    pub user_id: String,
    pub order_type: order_type::OrderType,
    pub order_execution_type: order_execution_type::OrderExecutionType,
    pub fill_or_kill: bool,
    pub price: u128,
    pub amount: u128,
    pub pair: order_pair::Pair,
    pub timestamp: u64
}

impl Order {
    pub fn new(user_id: &str, order_type: order_type::OrderType, order_execution_type: order_execution_type::OrderExecutionType, f_o_k: bool, price: u128, amount: u128, pair: &order_pair::Pair) -> Self {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                match uuid::Uuid::parse_str(pair.uuid.as_str()) {
                    Ok(uuid) => {
                        Order {
                            uuid: Uuid::new_v4(),
                            user_id: String::from(user_id),
                            order_type,
                            order_execution_type,
                            fill_or_kill: f_o_k,
                            price,
                            amount,
                            pair: order_pair::Pair::new(pair.price_ticker.as_str(), pair.ref_ticker.as_str(), uuid),
                            timestamp: n.as_secs()
                        }
                    },
                    Err(e) => panic!("Could not parse uuid from generated pair type: {}", e)
                }
            },
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    pub fn total_cost(&self) -> u128 {
        self.price * self.amount
    }
    pub fn serialize(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap();
        serialized
    }
    pub fn deserialize(json: &String) -> Self {
        let deserialized: Order = serde_json::from_str(json).unwrap();
        deserialized
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::order_type::OrderType;
    use crate::order::order_execution_type::OrderExecutionType;

    #[test]
    fn test_serde_order() {
        let pair = &order_pair::Pair::new("btc", "usd", uuid::Uuid::new_v4());

        let o: Order = Order::new("user", OrderType::BID, OrderExecutionType::LIMIT, true, 1000, 1000, pair);
        let serialized = o.serialize();
        let deserialized: Order = Order::deserialize(&serialized);

        assert_eq!(deserialized.user_id, o.user_id);

        match deserialized.order_type {
            OrderType::BID => { },
            _ => {
                assert_eq!(1, 2);
            }
        }

        match deserialized.order_execution_type {
            OrderExecutionType::LIMIT => { },
            _ => {
                assert_eq!(1, 2);
            }
        }

        assert_eq!(deserialized.fill_or_kill, o.fill_or_kill);
        assert_eq!(deserialized.price, o.price);
        assert_eq!(deserialized.amount, o.amount);
        assert_eq!(deserialized.pair.uuid, o.pair.uuid);

        assert_eq!(1, 1);
    }
}
