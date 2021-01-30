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
    use redis::connection;

    pub fn init_pubsub(conn: &mut connection::Connection) -> connection::PubSub {
        conn.as_pubsub()
    }
}