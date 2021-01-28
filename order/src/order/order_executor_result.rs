use std::time::SystemTime;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum OrderExecutorResult {
    ANNIHILATESEXISTING(u128, u128, u64),   //(amount, price, timestamp)
    PARTIALLYCLEARSEXISTING(u128, u128, u64),
    BOTHORDERSFILLEDEXACTLY(u128, u128, u64)
}

impl OrderExecutorResult {
    pub fn create(&self, price: u128, amount: u128) -> Self {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                match self {
                    OrderExecutorResult::ANNIHILATESEXISTING(_,_,_) => {
                        OrderExecutorResult::ANNIHILATESEXISTING (
                            price,
                            amount,
                            n.as_secs()
                            )
                    },
                    OrderExecutorResult::PARTIALLYCLEARSEXISTING(_,_,_) => {
                        OrderExecutorResult::PARTIALLYCLEARSEXISTING (
                            price,
                            amount,
                            n.as_secs()
                        )
                    },
                    OrderExecutorResult::BOTHORDERSFILLEDEXACTLY(_,_,_) => {
                        OrderExecutorResult::BOTHORDERSFILLEDEXACTLY (
                            price,
                            amount,
                            n.as_secs()
                        )
                    },
                }
            },
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
}