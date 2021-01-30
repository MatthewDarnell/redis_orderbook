use std::time::SystemTime;
use std::cmp::Ordering;
use crate::order::{Order, order_type};
use crate::order::order_executor_result::OrderExecutorResult;

pub trait Executor {
    fn can_execute_order(&self, order: &Order, existing_order: &Order) -> bool;
    fn execute_order(&self, new_order: &Order, existing_order: &Order) -> Result<OrderExecutorResult, String>;
}

pub enum OrderExecutor {
    LIMITEXECUTOR,
    MARKETEXECUTOR
}

impl Executor for OrderExecutor {
    fn can_execute_order(&self, order: &Order, existing_order: &Order) -> bool {
        match self {
            OrderExecutor::MARKETEXECUTOR => true,
            OrderExecutor::LIMITEXECUTOR => {
                match existing_order.order_type {
                    order_type::OrderType::ASK => {
                        if let order_type::OrderType::ASK = order.order_type {
                            return false;   //Cannot be the same order_type
                        }

                        if existing_order.price > order.price { return false; }
                    },
                    order_type::OrderType::BID => {
                        if let order_type::OrderType::BID = order.order_type {
                            return false;   //Cannot be the same order_type
                        }
                        if existing_order.price < order.price { return false; }
                    },
                    _ => { return false; },    //delete type doesn't execute at all
                }
                true
            }
        }
    }

    fn execute_order(&self, new_order: &Order, existing_order: &Order) -> Result<OrderExecutorResult, String> {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                match new_order.amount.cmp(&existing_order.amount) {
                    Ordering::Greater => { //New order completely clears existing order
                        Ok(OrderExecutorResult::ANNIHILATESEXISTING(existing_order.amount, existing_order.price, n.as_secs()))
                    },
                    Ordering::Less => { //New order Partially clears existing order
                        println!(" {} LESS THAN {}", &new_order.amount, &existing_order.amount);

                        Ok(OrderExecutorResult::PARTIALLYCLEARSEXISTING(new_order.amount, existing_order.price, n.as_secs()))
                    },
                    Ordering::Equal => { //Exactly filled, delete both orders
                        println!(" {} EQUAL TO {}", &new_order.amount, &existing_order.amount);

                        Ok(OrderExecutorResult::BOTHORDERSFILLEDEXACTLY(existing_order.amount, existing_order.price, n.as_secs()))
                    },
                }
            },
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
}
