extern crate redis;
use uuid::Uuid;
use std::str::FromStr;
use crate::order::Order;
use crate::order::order_type::OrderType;
use crate::order::store::order_execution_type_store::get_orders_by_price;
use redis::connection::Connection;
use redis::types::{redis_hash, redis_list, redis_set, redis_sorted_set};

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ListEntry {
    user_id: String,
    pub uuid: String
}

impl ListEntry {
    pub fn new(user_id: &str, uuid: &str) -> Self {
        ListEntry {
            user_id: String::from(user_id),
            uuid: String::from(uuid)
        }
    }
    pub fn serialize(&self) -> String {
        let serialized = serde_json::to_string(self).unwrap();
        serialized
    }
    pub fn deserialize(json: &String) -> Self {
        let deserialized: ListEntry = serde_json::from_str(json).unwrap();
        deserialized
    }
}


pub fn get_order_by_id(conn: &mut Connection, uuid: &Uuid) -> Option<String> {
    let uuid = uuid.to_string();
    //TODO: parse response into order object
    match redis_hash::hget(conn, "orders", &uuid) {
        Ok(order) => {
            let order: String = order;
            Some(order)
        },
        Err(e) => {
            println!("Error Getting Order by Id<{}> -- {}", uuid, e);
            None
        }
    }
}

pub fn get_open_orders_for_ticker(conn: &mut Connection, user_id: &str, ticker: &str) -> Option<u128> {
    let mut key = String::from(user_id);
    key.push('-');
    key.push_str(ticker);
    match redis_hash::hget(conn, "user_open_order_sums", key.as_str()) {
        Ok(res) => {
            let res: String = res;
            let res: u128 = res.parse().unwrap_or_else(|_| 0);
            Some(res)
        },
        Err(_) => Some(0)
    }
}

fn increment_user_open_order_balance<T: std::fmt::Display>(conn: &mut Connection, order: &Order, delta: T) {
    let ticker = match &order.order_type {
        OrderType::BID => &order.pair.ref_ticker,
        OrderType::ASK => &order.pair.price_ticker,
        _ => panic!("Trying to increment user open order balance with delete order type...")
    };
    let mut value = String::from(&order.user_id);
    value.push('-');
    value.push_str(ticker);
    match redis_hash::hincrby(conn, "user_open_order_sums", &value, delta.to_string().as_str()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to increment open order sums by {} -- {}", delta.to_string().as_str(), e)
    }
}

fn update_sums<T: std::fmt::Display>(conn: &mut Connection, order_type: &OrderType, pair_id: &String, price: u128, amount: T) {   //redis constraint is i64
    let mut value = String::from(pair_id);
    value.push('-');
    value.push_str(price.to_string().as_str());

    let sums_field = match order_type {
        OrderType::BID => "bids-sums",
        OrderType::ASK => "asks-sums",
        OrderType::DELETE => panic!("Can't get sum of delete orders")
    };

    redis_hash::hincrby(conn, sums_field, value.as_str(), amount.to_string().as_str());

    let sum_amount: u64 = redis_hash::hget(conn, sums_field, value.as_str()).unwrap();
    if sum_amount < 1 {
        redis_hash::hdel(conn, sums_field, value.as_str());
    }

}

pub fn update_order_amount(conn: &mut Connection, order: &mut Order, amount_to_update: i128) {
    let current_total_cost = order.amount * order.price;
    if amount_to_update < 0 {
        match amount_to_update.checked_abs() {
            Some(abs) => {
                let abs = abs as u128;
                order.amount = order.amount.wrapping_sub(abs);
            },
            None => panic!("Could not update order by amount {}", amount_to_update)
        }
    } else {
        order.amount = order.amount.wrapping_add(amount_to_update as u128);
    }

    let new_total_cost = order.amount * order.price;
    let order: &Order = order;
    let serialized = order.serialize();
    redis_hash::hset(conn, "orders", &order.uuid.to_string(), &serialized);
    let subtracted: i128 = new_total_cost as i128 - current_total_cost as i128;
    match order.order_type {
        OrderType::BID => increment_user_open_order_balance(conn, order, subtracted),
        OrderType::ASK => increment_user_open_order_balance(conn, order, amount_to_update),
        OrderType::DELETE => panic!("Cannot update delete order amount type")
    }
    update_sums(conn, &order.order_type, &order.pair.uuid, order.price, amount_to_update);
}

fn create_bid(conn: &mut Connection, order: &Order, list_entry: &ListEntry) {
    let pair_uuid = &order.pair.uuid.to_string();
    let price = order.price;

    let mut key = String::from("BIDS-");
    key.push_str(pair_uuid);
    key.push('-');
    key.push_str(price.to_string().as_str());

    redis_list::rpush(conn, key.as_str(), list_entry.serialize());
    let mut sorted_set_key = String::from("bids-");
    sorted_set_key.push_str(pair_uuid);
    redis_sorted_set::zadd(conn, sorted_set_key.as_str(), key.as_str(), price.to_string().as_str());

     increment_user_open_order_balance(conn, order, order.total_cost() as i128);
}

fn create_ask(conn: &mut Connection, order: &Order, list_entry: &ListEntry) {
    let pair_uuid = &order.pair.uuid.to_string();
    let price = order.price;

    let mut key = String::from("ASKS-");
    key.push_str(pair_uuid);
    key.push('-');
    key.push_str(price.to_string().as_str());

    redis_list::rpush(conn, key.as_str(), list_entry.serialize());
    let mut sorted_set_key = String::from("asks-");
    sorted_set_key.push_str(pair_uuid);
    redis_sorted_set::zadd(conn, sorted_set_key.as_str(), key.as_str(), price.to_string().as_str());

    increment_user_open_order_balance(conn, order, order.amount as i128);
}

//Add a new order to uuid=>Order hash table
//and the uuid to the User=>uuid hash table
pub fn create_order(conn: &mut Connection, order: &Order) {
    let uuid = &order.uuid.to_string();
    let user_id = &order.user_id;
    let mut key: String = String::from("users-orders-");
    key.push_str(user_id);

    redis_set::sadd(conn, key.as_str(), uuid);
    redis_hash::hset(conn, "orders", uuid, order.serialize());

    let list_entry = ListEntry::new(user_id, uuid);

    match order.order_type {
        OrderType::BID => {
            create_bid(conn, order, &list_entry);
        },
        OrderType::ASK => {
            create_ask(conn, order, &list_entry);
        },
        _ => panic!("Trying to create Delete Order...")
    }
    update_sums(conn, &order.order_type, &order.pair.uuid, order.price, order.amount);
}

//Delete an order
pub fn delete_order(conn: &mut Connection, order: &Order) {
    redis_hash::hdel(conn, "orders", order.uuid.to_string().as_str());
    let list_element_to_remove: ListEntry = ListEntry::new(order.user_id.as_str(), order.uuid.to_string().as_str());
    let mut list_key = match order.order_type {
        OrderType::ASK => String::from("ASKS-"),
        OrderType::BID => String::from("BIDS-"),
        _ => panic!("Cannot delete this order")
    };
    list_key.push_str(order.pair.uuid.as_str());
    list_key.push('-');
    list_key.push_str(order.price.to_string().as_str());

    redis_list::lrem(conn, list_key.as_str(), 0, list_element_to_remove.serialize());

    let length_remaining = redis_list::llen(conn, list_key.as_str()).unwrap();
    if length_remaining < 1 {   //delete the list
        redis::connection::del(conn, list_key.as_str());
    }

    let mut list_key = String::from("users-orders-");
    list_key.push_str(order.user_id.as_str());
    redis_set::srem(conn, list_key.as_str(), order.uuid.to_string().as_str());

    update_sums(conn, &order.order_type, &order.pair.uuid, order.price, 0-order.amount as i128);

    match order.order_type {
        OrderType::BID => increment_user_open_order_balance(conn, &order, 0 - ( order.amount * order.price ) as i128),
        OrderType::ASK => increment_user_open_order_balance(conn, &order, 0 - order.amount as i128),
        OrderType::DELETE => panic!("Cannot delete this order")
    }
}

pub fn get_orders_by_user_id(conn: &mut Connection, user_id: &str) -> Option<Vec<Order>> {
    let mut key = String::from("users-orders-");
    key.push_str(user_id);
    match redis_set::smembers(conn, key.as_str()) {
        Ok(res) => {
            let res: Vec<String> = res;
            let mut orders: Vec<Order> = Vec::new();
            for r in res {
                let id = uuid::Uuid::from_str(r.as_str()).unwrap();
                match get_order_by_id(conn, &id) {
                    Some(order_string) => {
                        let retrieved_order: Order = Order::deserialize(&order_string);
                        orders.push(retrieved_order);
                    },
                    None => {
                        panic!("Could not retrieve order {}", r.as_str());
                    }
                }
            }
            Some(orders)
        },
        Err(e) => panic!("Error getting orders by user id {} -- {}", user_id, e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::order_type::OrderType;
    use crate::order::order_pair;
    use crate::order::order_execution_type::OrderExecutionType;
    use crate::order::store::order_execution_type_store;

    #[test]
    fn test_create_retrieve_order() {
        let pair = &order_pair::Pair::new("btc", "usd", uuid::Uuid::new_v4());

        let o: Order = Order::new("test_user_id", OrderType::BID, OrderExecutionType::LIMIT, true, 1000, 1001, pair);

        let serialized = o.serialize();
        let deserialized: Order = Order::deserialize(&serialized);
        if let mut conn = redis::connection::get_connection(None).unwrap() {
            create_order(&mut conn, &o);
            match get_order_by_id(&mut conn, &o.uuid) {
                Some(res) => {
                    let retrieved_order: Order = Order::deserialize(&res);
                    //assert_eq!(o.uuid.to_string(), retrieved_order.uuid.to_string());
                },
                None => {
                    println!("Could not retrieve order {}", &o.uuid.to_string());
                }
            }

            match get_orders_by_user_id(&mut conn, "test_user_id") {
                Some(res) => {
                    //assert_eq!(res.len(), 1);
                    let order_id = &res[0].uuid.to_string();
                    //assert_eq!(order_id, &o.uuid.to_string());
                },
                None => {
                    println!("Could not retreive orders");
                    assert_eq!(1, 2);
                }
            }
/*

            match get_open_orders_for_ticker(&mut conn, "test_user_id", "btc") {
                Some(value) => assert_eq!(value, 1000000),
                None => assert_eq!(1,2)
            }
*/
            order_execution_type_store::get_orders_by_price(&mut conn, &OrderType::BID, pair, 0, 1000000000);


        }

    }
}
