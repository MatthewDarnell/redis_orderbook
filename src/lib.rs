extern crate redis;
pub use redis::{connection, types::redis_key, types::redis_hash};
extern crate order;
extern crate uuid;
extern crate trade;
use uuid::Uuid;

use order::order::{store, order_type, order_execution_type};
use order::order::order_pair::Pair;
use order::order::orderbook;
pub use redis::connection::RedisError;
use order::order::store::order_pair_store::{add_new_pair, get_pairs, get_pair_by_id};
use redis::connection::Connection;


pub fn init(ip_addr: &str) -> Result<Connection, RedisError> {
    match ip_addr {
        "" => connection::get_connection(None),
        _ => connection::get_connection(Some(ip_addr))
    }
}

pub fn get_orderbook(conn: &mut Connection, pair_id: &str, depth: u32) -> Result<String, String> {
    order::get_orderbook(conn, String::from(pair_id), depth)
}


pub fn place_trade(
    conn: &mut Connection,
    publish_completed_trades: bool,
    user_id: &str,
    order_type: &str,
    order_execution_type: &str,
    fill_or_kill: bool,
    price: u128,
    amount: u128,
    pair_id: &str
) -> Result<String, String> {
   let order_type: order::order::order_type::OrderType = match order_type.to_ascii_uppercase().as_str() {
       "BID" => order_type::OrderType::BID,
       "ASK" => order_type::OrderType::ASK,
       _ => return Err("Invalid Order Type".to_string())
   };

    let order_execution_type: order::order::order_execution_type::OrderExecutionType = match order_execution_type.to_ascii_uppercase().as_str() {
        "LIMIT" => order_execution_type::OrderExecutionType::LIMIT,
        "MARKET" => order_execution_type::OrderExecutionType::MARKET,
        _ => return Err("Invalid Order Execution Type".to_string())
    };

    let pair_id = match uuid::Uuid::parse_str(pair_id) {
        Ok(id) => id,
        Err(_) => return Err("Invalid Pair Uuid".to_string())
    };

    let pair = match get_pair_by_id(conn, pair_id) {
        Some(pair) => pair,
        None => return Err("Invalid Pair Id".to_string())
    };

    if amount == 0 {
        return Err("Invalid Amount".to_string());
    }
    if price == 0 {
        return Err("Invalid Price".to_string());
    }

    let mut order = order::order::Order::new(
        user_id,
        order_type,
        order_execution_type,
        fill_or_kill,
        price,
        amount,
        &pair
    );

    trade::trade::place_trade(conn, publish_completed_trades, &mut order);

    Ok("ok".to_string())
}

pub fn get_user_open_order_sum(conn: &mut Connection, user_id: &str) {
    get_user_open_order_sum(conn, user_id)
}

pub fn create_pair(conn: &mut Connection, price_ticker: &str, ref_ticker: &str) {
    let pair = Pair::new(price_ticker, ref_ticker, uuid::Uuid::new_v4());
    add_new_pair(conn, &pair)
}

pub fn get_all_pairs(conn: &mut Connection) -> Result<String, String> {
    let pairs = get_pairs(conn);
    if pairs.is_empty() {
        return Ok("".to_string());
    }
    let mut ret_val: String = String::from("[");
    for p in &pairs {
        ret_val.push_str(p.serialize().as_str());
        ret_val.push_str(", ");
    }
    ret_val.pop();
    ret_val.pop();
    ret_val.push(']');
    Ok(ret_val)
}



pub mod redis_pubsub {
    use std::thread;
    use std::time::Duration;
    use redis::connection;
    use super::init;


    use serde::Deserialize;
    use serde_json::Value;
    use order::order::order_type::OrderType;
    use order::order::order_execution_type::OrderExecutionType;
    use order::order::store::order_pair_store::get_pair_by_id;
    use redis::connection::Commands;
    use order::order::store::order_store::delete_order;

    #[derive(Deserialize, Debug)]
    struct IncomingOrder {
        user_id: String,
        order_type: String,
        order_execution_type: String,
        fill_or_kill: bool,
        price: u64,
        amount: u64,
        pair: String
    }

    impl IncomingOrder {
        pub fn deserialize(json: &String) -> Self {
            let deserialized: IncomingOrder = serde_json::from_str(json).unwrap_or_else(|_| panic!("Unable to deserialize IncomingOrder"));
            deserialized
        }
    }

    pub fn init_pubsub(conn: &mut connection::Connection) -> connection::PubSub {
        conn.as_pubsub()
    }

    pub fn init_listening_for_orders(order_subscription_channel: &'static str, created_order_publishing_channel: &'static str) {
        let handle = thread::spawn(move || {

            let mut conn = init("").unwrap();
            let mut pubsub = init("").unwrap();
            let mut pub_sub: connection::PubSub = pubsub.as_pubsub();

            println!("Starting Redis Order Subscriber Thread");
            pub_sub.subscribe(order_subscription_channel).unwrap_or_else(|_| {
                println!("Failed to subscribe to channel {}. Shutting down Thread", order_subscription_channel);
            });
            println!("Listening for incoming orders on channel <{}>", order_subscription_channel);
            println!("Publishing created orders on channel <{}>", created_order_publishing_channel);
            println!("Submit Order For Processing:\tredis publish {} \"{{ \"user_id\": \"user_uuid\", \"order_type\": \"BID/ASK\", \"order_execution_type\": \"LIMIT/MARKET\", \"fill_or_kill\": true/false, \"price\": u64, \"amount\": u64, \"pair\": \"pair_uuid\" }}\"", order_subscription_channel);
            println!("Cancel An Open Order:\tredis publish {} \"{{\"order_type\": \"DELETE\", \"uuid\": \"order_uuid\"}}\"", order_subscription_channel);
            loop {
                match pub_sub.get_message() {
                    Ok(msg) => {
                        match msg.get_payload() {
                            Ok(payload) => {
                                let payload: String = payload;
                                if payload.as_str() == "quit" {
                                    println!("Received Quit notification, shutting down");
                                    break;
                                }

                                let mut incoming_order: Value = serde_json::from_str(payload.as_str()).unwrap();

                                let mut ord_type: String = incoming_order["order_type"].as_str().unwrap().to_string();

                                if ord_type.trim() == "DELETE" {
                                    let order_id: String = incoming_order["uuid"].as_str().unwrap().to_string();
                                    match order::order::store::order_store::get_order_by_id(&mut conn, &uuid::Uuid::parse_str(order_id.as_str()).unwrap()) {
                                        Some(o) => {
                                            let order = order::order::Order::deserialize(&o);
                                            delete_order(&mut conn, &order);
                                        },
                                        None => panic!("No such order {}", order_id.as_str())
                                    }
                                    continue;
                                }

                                let incoming_order: IncomingOrder = serde_json::from_value(incoming_order).unwrap();

                                let order_type = match incoming_order.order_type.to_string().to_ascii_uppercase().as_str() {
                                    "BID" => OrderType::BID,
                                    "ASK" => OrderType::ASK,
                                    _ => {
                                        println!("Malformed Order! No type {}", incoming_order.order_type);
                                        continue;
                                    },
                                };


                                let order_execution_type = match incoming_order.order_execution_type.to_string().to_ascii_uppercase().as_str() {
                                    "LIMIT" => OrderExecutionType::LIMIT,
                                    "MARKET" => OrderExecutionType::MARKET,
                                    _ => {
                                        println!("Malformed Order! No type {}", incoming_order.order_execution_type);
                                        continue;
                                    },
                                };

                                let pair = uuid::Uuid::parse_str(incoming_order.pair.to_string().as_str()).unwrap();
                                let pair = match get_pair_by_id(&mut  conn, pair) {
                                    Some(pair) => pair,
                                    None => {
                                        println!("Invalid Pair {}", pair);
                                        continue;
                                    }
                                };

                                let user_id: String = incoming_order.user_id;
                                let fill_or_kill: bool = incoming_order.fill_or_kill;
                                let price: u64 = incoming_order.price;
                                let amount: u64 = incoming_order.amount;

                                let mut ord = order::order::Order::new(
                                    user_id.as_str(),
                                    order_type,
                                    order_execution_type,
                                    fill_or_kill,
                                    price as u128,
                                    amount as u128,
                                    &pair
                                );

                                match trade::trade::place_trade(&mut conn, true, &mut ord) {
                                    Some(uuid) => {
                                        let mut json_string = String::from("{\"user_id\": \"");
                                        json_string.push_str(user_id.as_str());
                                        json_string.push_str("\", \"uuid\": \"");
                                        json_string.push_str(uuid.as_str());
                                        json_string.push_str("\"}");

                                        redis::types::redis_pubsub::publish(&mut conn, created_order_publishing_channel, json_string.as_str());
                                    },
                                    None => {}
                                }
                            },
                            Err(e) => println!("Failed to read message payload! {}", e)
                        }
                    },
                    Err(e) => println!("Failed to read message! {}", e)
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
        handle.join().unwrap();
        println!("Stopping Redis Order Subscriber Thread");
    }



}