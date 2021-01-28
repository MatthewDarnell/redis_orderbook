extern crate redis;
use std::time::SystemTime;
use uuid::Uuid;
use crate::order::Order;
use crate::order::order_type::OrderType;
use crate::order::order_executor_result::OrderExecutorResult;
use redis::connection::Connection;
use redis::types::{redis_hash, redis_key, redis_list, redis_set, redis_sorted_set};
use self::redis::connection::RedisResult;
use crate::order::order_pair::Pair;


pub fn add_new_pair(conn: &mut Connection, pair: Pair) {
    let serialized_pair = pair.serialize();
    redis_set::sadd(conn, "pairs", serialized_pair.as_str());
}

pub fn is_valid_pair(conn: &mut Connection, pair: &Pair) -> bool {
    let pair = pair.serialize();
    match redis_set::sismember(conn, "pairs", pair) {
        Ok(res) => {
            let res: bool = res;
            res
        },
        Err(e) => {
            panic!("Could not retrieve pair sismember: {}", e);
        }
    }
}

pub fn get_pairs(conn: &mut Connection) -> Vec<Pair> {
    match redis_set::smembers(conn, "pairs") {
        Ok(res) => {
            let res: Vec<String> = res;
            let mut result: Vec<Pair> = Vec::new();
            for r in res {
                result.push(Pair::deserialize(&r));
            }
            result
        },
        Err(e) => {
            panic!("Could not retrieve pairs: {}", e);
        }
    }

}

pub fn get_pair_by_id(conn: &mut Connection, pair_id: Uuid) -> Option<Pair> {
    let all_pairs = get_pairs(conn);
    for pair in all_pairs {
        if pair.uuid == pair_id.to_string() {
            return Some(pair);
        }
    }
    None
}