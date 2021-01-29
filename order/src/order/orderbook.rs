extern crate redis;
use crate::order::Order;
use crate::order::order_type::OrderType;
use redis::connection::Connection;
use redis::types::{redis_hash};
use crate::order::store::order_execution_type_store::{get_orders_by_price, get_orders_by_price_index};


pub fn get_sum_of_orders_for_price_point(conn: &mut Connection, pair: &String, price: u128) -> String {
    let mut field: String = String::from(pair);
    field.push('-');
    field.push_str(price.to_string().as_str());
    println!("hget sums {}", field.as_str());
    match redis_hash::hget(conn, "sums", field.as_str()) {
        Ok(res) => {
            let res: String = res;
            res
        },
        Err(_) => {
            //panic!("Couldn't get sum of orders for price point {} -- {}", price, e);
            "0".to_string()
        }
    }
}

pub fn get_matching_market_orders_for_execution(conn: &mut Connection, order: &Order) -> Vec<Order> {
    let mut index: isize = -1;
    let mut all_orders: Vec<Order> = Vec::new();

    let mut order_amount_count: u128 = 0;

    while order_amount_count < order.amount {
        index = index + 1;
        let mut orders = match &order.order_type {
            OrderType::ASK => {
                get_orders_by_price_index(conn, &OrderType::BID, &order.pair, index, index)
            },
            OrderType::BID => {
                get_orders_by_price_index(conn, &OrderType::ASK, &order.pair, index, index)
            },
            _ => panic!("Cannot get Delete type orders from orderbook")
        };

        if orders.is_empty() {
            break;
        }

        for order in &orders {
            order_amount_count += order.amount;
        }

        all_orders.append(&mut orders);
    }
        all_orders
}

pub fn get_matching_limit_orders_for_execution(conn: &mut Connection, order: &Order) -> Vec<Order> {
    match &order.order_type {
        OrderType::ASK => {
            get_orders_by_price(conn, &OrderType::BID, &order.pair, order.price.to_string().as_str(), "+inf")
        },
        OrderType::BID => {
            get_orders_by_price(conn, &OrderType::ASK, &order.pair, 0, order.price.to_string().as_str())
        },
        _ => panic!("Cannot get Delete type orders from orderbook")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::order_type::OrderType;
    use crate::order::order_execution_type::OrderExecutionType;

    fn set_up_test() {
        let pair = &order_pair::Pair::new("btc", "usd", uuid::Uuid::new_v4());

        Order::new("user1", OrderType::BID, OrderExecutionType::LIMIT, false, 1, 10, pair);
        Order::new("user2", OrderType::BID, OrderExecutionType::LIMIT, true, 2, 10, pair);
        Order::new("user3", OrderType::BID, OrderExecutionType::LIMIT, true, 3, 10, pair);

        Order::new("user4", OrderType::ASK, OrderExecutionType::LIMIT, false, 3, 10, pair);
        Order::new("user5", OrderType::ASK, OrderExecutionType::LIMIT, true, 2, 10, pair);
        Order::new("user6", OrderType::ASK, OrderExecutionType::MARKET, true, 1, 10, pair);

    }
    fn clean_up_test() {

    }


    #[test]
    fn test_get_order_by_id() {
        let pair = &order_pair::Pair::new("btc", "usd", uuid::Uuid::new_v4());
        let o: Order = Order::new("user", OrderType::BID, OrderExecutionType::LIMIT, true, 1000, 1000, pair);
        let serialized = o.serialize();
        let deserialized: Order = Order::deserialize(&serialized);

        assert_eq!(deserialized.user_id, o.user_id);

        match deserialized.order_type {
            OrderType::BID => { },
            _ => {
                assert_eq!(1, 2);
            }
        }

        match deserialized.order_execution_type {
            OrderExecutionType::LIMIT => { },
            _ => {
                assert_eq!(1, 2);
            }
        }

        assert_eq!(deserialized.fill_or_kill, o.fill_or_kill);
        assert_eq!(deserialized.price, o.price);
        assert_eq!(deserialized.amount, o.amount);
        assert_eq!(deserialized.pair.uuid, o.pair.uuid);

        assert_eq!(1, 1);
    }
}
