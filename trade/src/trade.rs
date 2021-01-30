extern crate order;
extern crate redis;
use std::time::SystemTime;
use redis::connection::Connection;
use redis::connection::PubSub;
use order::order::Order;
use order::order::order_type::OrderType;
use order::order::order_executor::{ Executor, OrderExecutor };
use order::order::order_execution_type::OrderExecutionType;
use order::order::orderbook;
use self::order::order::order_executor_result::OrderExecutorResult;
use order::order::store::order_store;
use order::order::store::order_pair_store;
use self::order::order::store::order_store::{delete_order, create_order};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct TradePlaced {
    pair_id: String,
    pair: String,
    execution_price: String,
    filled_amount: String,
    side: String,
    bid_user_id: String,
    ask_user_id: String,
    bid_order_id: String,
    ask_order_id: String,
    timestamp: String
}


impl TradePlaced {
    pub fn new(pair_id: String, pair: String, execution_price: String, filled_amount: String, side: String, bid_user_id: String, ask_user_id: String, bid_order_id: String, ask_order_id: String, timestamp: String) -> Self {
        TradePlaced {
            pair_id,
            pair,
            execution_price,
            filled_amount,
            side,
            bid_user_id,
            ask_user_id,
            bid_order_id,
            ask_order_id,
            timestamp
        }
    }
    pub fn serialize(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap();
        serialized
    }
    pub fn deserialize(json: &String) -> Self {
        let deserialized: TradePlaced = serde_json::from_str(json).unwrap();
        deserialized
    }
}

pub fn place_trade(conn: &mut Connection, publish_completed_trades: bool, o: &mut Order) -> Option<String> {
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

    let mut return_uuid: bool = false;  //return the uuid if this order is created

    loop {
        if o.amount <= 0 {
            break;
        }

        if existing_orders.is_empty() {
            //place_order of new order remainder if last remaining existing order
            match o.order_execution_type {
                OrderExecutionType::LIMIT => {
                    create_order(conn, o);
                    return_uuid = true;
                },
                _ => {} //MARKET orders are dropped if they cannot be filled
            }
            break;
        }

        let mut existing_order = existing_orders.remove(0);

        let mut trade_filled_amount = 0;

        if !order_executor.can_execute_order(o, &existing_order) {
            continue;
        }

        match order_executor.execute_order(o, &existing_order).unwrap() {
            OrderExecutorResult::PARTIALLYCLEARSEXISTING(amount, price, timestamp) => {
                o.amount = 0;
                trade_filled_amount = amount;
                order_store::update_order_amount(conn, &mut existing_order, 0-amount as i128);
            },
            OrderExecutorResult::BOTHORDERSFILLEDEXACTLY(amount, price, timestamp) => {
                o.amount  = 0;
                trade_filled_amount = amount;
                delete_order(conn, &existing_order);
            },
            OrderExecutorResult::ANNIHILATESEXISTING(amount, price, timestamp) => {
                o.amount -= amount;
                trade_filled_amount = amount;
                delete_order(conn, &existing_order);
            }
        }

        //Store trade in redis set
        let (side, bid_user_id, bid_order_id, ask_user_id, ask_order_id) = {
            match o.order_type {
                OrderType::BID => ("BID".to_string(), String::from(&o.user_id), String::from(&o.uuid.to_string()), String::from(&existing_order.user_id), String::from(&existing_order.uuid.to_string())),
                OrderType::ASK => ("ASK".to_string(), String::from(&existing_order.user_id), String::from(&existing_order.uuid.to_string()), String::from(&o.user_id), String::from(&o.uuid.to_string())),
                _ => panic!("Invalid order type to log")
            }
        };

        let pair_id = o.pair.uuid.to_string();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();
        let execution_price = String::from(&existing_order.price.to_string());

        let pair = uuid::Uuid::parse_str(pair_id.as_str()).unwrap();

        let pair = order_pair_store::get_pair_by_id(conn, pair).unwrap().serialize();


        let filled_amount: String = trade_filled_amount.to_string();

        let side = side;

        let trade = TradePlaced::new(
            pair_id,
            pair,
            execution_price,
            filled_amount,
            side,
            bid_user_id,
            ask_user_id,
            bid_order_id,
            ask_order_id,
            timestamp
        ).serialize();

        redis::types::redis_set::sadd(conn, "trades_completed", trade.as_str());

        if publish_completed_trades {
            redis::types::redis_pubsub::publish(conn, "trades_completed", &trade);
        }

    }
    if return_uuid {
        Some(o.uuid.to_string())
    } else {
        None
    }
}

