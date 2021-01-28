extern crate order;
extern crate redis;
use redis::connection::Connection;
use order::order::Order;
use order::order::order_type::OrderType;
use order::order::order_executor::{ Executor, OrderExecutor };
use order::order::order_execution_type::OrderExecutionType;
use order::order::orderbook;
use self::order::order::order_executor_result::OrderExecutorResult;

pub fn place_order(conn: &mut Connection, o: &Order) {
    let side_to_get = match o.order_type {
        OrderType::BID => OrderType::ASK,
        OrderType::ASK => OrderType::BID,
        _ => panic!("Invalid Delete to place order!")
    };

    let order_executor = match o.order_execution_type {
        OrderExecutionType::MARKET => OrderExecutor::MARKETEXECUTOR,
        OrderExecutionType::LIMIT=> OrderExecutor::LIMITEXECUTOR
    };

    let mut existing_orders = match o.order_execution_type {
        OrderExecutionType::MARKET => orderbook::get_matching_market_orders_for_execution(conn, o),
        OrderExecutionType::LIMIT => orderbook::get_matching_limit_orders_for_execution(conn, o)
    };

    let mut amount_remaining: u128 = o.amount;

    loop {
        if amount_remaining <= 0 {
            break;
        }

        if existing_orders.is_empty() {
            let order_amount_delta = o.amount;
            //TODO: place_new_order and break out
        }

        let existing_order = existing_orders.remove(0);

        if !order_executor.can_execute_order(o, &existing_order) {
            continue;
        }

        match order_executor.execute_order(o, &existing_order).unwrap() {
            OrderExecutorResult::PARTIALLYCLEARSEXISTING(amount, price, timestamp) => {
                amount_remaining -= amount;
                //place_trade
                //delete_order
            },
            OrderExecutorResult::BOTHORDERSFILLEDEXACTLY(amount, price, timestamp) => {
                amount_remaining = 0;
                //place_trade
                //delete_order
            },
            OrderExecutorResult::ANNIHILATESEXISTING(amount, price, timestamp) => {
                amount_remaining = 0;
                //place_trade
                if existing_orders.len() == 0 {
                    //place_order of new order remainder if last remaining existing order
                }
            }
        }



        break;
    }


}