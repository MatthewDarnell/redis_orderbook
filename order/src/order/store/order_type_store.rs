extern crate redis;
use std::time::SystemTime;
use std::cmp::Ordering;
use uuid::Uuid;
use crate::order::Order;
use crate::order::order_type::OrderType;
use crate::order::order_executor_result::OrderExecutorResult;
use redis::connection::Connection;
use redis::types::{redis_hash, redis_key, redis_list, redis_set, redis_sorted_set};
use self::redis::connection::{RedisResult, ToRedisArgs};


fn get_key_by_order_type(order_type: &OrderType) -> String {
    match order_type {
        OrderType::ASK => "ASK".to_string(),
        OrderType::BID => "BID".to_string(),
        _ => panic!("Invalid Order in Orderbook Exection (Delete type being executed?)")
    }
}

pub fn get_sorted_set_for_order_type(conn: &mut Connection, order_type: OrderType, depth: isize, desc: bool) -> RedisResult<Vec<String>> {
    let key = get_key_by_order_type(&order_type);
    match desc {
        true => redis_sorted_set::zrevrange(conn, key, 0, depth),
        false => redis_sorted_set::zrange(conn, key, 0, depth)
    }
}


pub fn get_range_sorted_set_for_order_type<T: ToRedisArgs, V: ToRedisArgs>(conn: &mut Connection, key: &str, min: T, max: V, desc: bool) -> RedisResult<Vec<String>> {
    match desc {
        true => redis_sorted_set::zrevrangebyscore(conn, key, max, min),
        false => redis_sorted_set::zrangebyscore(conn, key, min, max)
    }
}

pub fn get_range_sorted_set_by_index(conn: &mut Connection, key: &str, start: isize, end: isize, desc: bool) -> RedisResult<Vec<String>> {
    match desc {
        true => redis_sorted_set::zrevrange(conn, key, end, start),
        false => redis_sorted_set::zrange(conn, key, start, end)
    }
}