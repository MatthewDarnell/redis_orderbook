extern crate redis;
use std::cmp::Ordering;
use crate::order::Order;
use crate::order::order_type::OrderType;
use crate::order::order_execution_type::OrderExecutionType;
use redis::connection::Connection;
use redis::types::{redis_hash, redis_list};
use self::redis::connection::ToRedisArgs;
use self::redis::connection;
use crate::order::order_pair::Pair;
use crate::order::store::order_store;
use crate::order::store::order_type_store;


fn get_orders_from_uuid_list(conn: &mut Connection, orders: &Vec<String>) -> Vec<Order> {
    orders.into_iter().flat_map(|value| { //BIDS-f0385e73-6f35-4ddc-8bcf-5c3dd8b2941e-1000
        let fifo_orders: Vec<String> = redis_list::lrange(conn, value.as_str(), 0, -1).unwrap();
        match fifo_orders.len().cmp(&1) {
            Ordering::Greater => {
                match redis_hash::hmget(
                    conn,
                    "orders",
                    fifo_orders
                        .into_iter()
                        .map(|ord| (order_store::ListEntry::deserialize(&ord)).uuid)
                        .collect::<Vec<String>>()
                ) {
                    Ok(e) => {
                        let e: Vec<String> = e;
                        let values: Vec<Order> = e.into_iter().map(|ord| Order::deserialize(&ord)).collect();
                        values

                    },
                    Err(e) => panic!("error greater was {}", e)
                }
            },
            Ordering::Equal => {
                match redis_hash::hmget(
                    conn,
                    "orders",
                    order_store::ListEntry::deserialize(&fifo_orders[0]).uuid
                ) {
                    Ok(e) => {
                        let e: String = e;
                        let mut values: Vec<Order> = Vec::new();
                        values.push(Order::deserialize(&e));
                        values

                    },
                    Err(e) => panic!("error equal was {}", e)
                }
            },
            Ordering::Less => { //no orders in orderbook :(
                let values: Vec<Order> = Vec::new();
                values
            }
        }
    }).collect()
}

pub fn get_orders_by_price_index(conn: &mut Connection, order_type: &OrderType, pair: &Pair, start: isize, end: isize) -> Vec<Order> {
    let mut side_to_get: String = {
        match order_type {
            OrderType::BID => String::from("bids-"),
            OrderType::ASK => String::from("asks-"),
            _ => panic!("Attempting to retrieve Delete orders by price...")
        }
    };
    side_to_get.push_str(&pair.uuid.to_string());

    let orders;

    match order_type {
        OrderType::BID => { //zrange bids-f0385e73-6f35-4ddc-8bcf-5c3dd8b2941e 0 100000000
            orders = order_type_store::get_range_sorted_set_by_index(conn, &side_to_get.as_str(), start, end, true).unwrap();
        },
        OrderType::ASK => {
            orders = order_type_store::get_range_sorted_set_by_index(conn, &side_to_get.as_str(), start, end, false).unwrap();
        },
        _ => panic!("Can't query Delete orders by price...")
    }

    get_orders_from_uuid_list(conn, &orders)
}

pub fn get_orders_by_price<T: ToRedisArgs, V: ToRedisArgs>(conn: &mut Connection, order_type: &OrderType, pair: &Pair, min: T, max: V) -> Vec<Order> {
    let mut side_to_get: String = {
        match order_type {
            OrderType::BID => String::from("bids-"),
            OrderType::ASK => String::from("asks-"),
            _ => panic!("Attempting to retrieve Delete orders by price...")
        }
    };
    side_to_get.push_str(&pair.uuid.to_string());

    let orders;

    match order_type {
        OrderType::BID => { //zrange bids-f0385e73-6f35-4ddc-8bcf-5c3dd8b2941e 0 100000000
            orders = order_type_store::get_range_sorted_set_for_order_type(conn, &side_to_get.as_str(), min, max, true).unwrap();
        },
        OrderType::ASK => {
            orders = order_type_store::get_range_sorted_set_for_order_type(conn, &side_to_get.as_str(), min, max, false).unwrap();
        },
        _ => panic!("Can't query Delete orders by price...")
    }

    get_orders_from_uuid_list(conn, &orders)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_orders_from_uuid_list() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let list = "redis_list_test_key";
            let pair = &Pair::new("btc", "usd", uuid::Uuid::new_v4());
            let a: Order = Order::new("user1", OrderType::BID, OrderExecutionType::LIMIT, true, 1000, 1000, pair);
            let b: Order = Order::new("user2", OrderType::BID, OrderExecutionType::LIMIT, true, 1000, 1000, pair);

            let uuids_to_request: Vec<String> = vec![
                String::from("BIDS-") + pair.uuid.as_str() + "-1000",
            ];

            order_store::create_order(&mut conn, &a);
            order_store::create_order(&mut conn, &b);

            let orders: Vec<Order> = get_orders_from_uuid_list(&mut conn, &uuids_to_request);

            assert_eq!(orders.len(), 2);

            assert_eq!(orders[0].uuid.to_string(), a.uuid.to_string());
            assert_eq!(orders[1].uuid.to_string(), b.uuid.to_string());

        } else {
            assert_eq!(1, 2);
        }
    }
}