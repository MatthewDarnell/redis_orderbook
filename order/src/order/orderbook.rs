extern crate redis;
use crate::order::Order;
use crate::order::order_type::OrderType;
use redis::connection::Connection;
use redis::types::{redis_hash};
use crate::order::store::order_execution_type_store::{get_orders_by_price, get_orders_by_price_index};


pub fn get_user_open_order_sum(conn: &mut Connection, user_id: &str, ticker: &str) -> String {
    let mut field = String::from(user_id);
    field.push('-');
    field.push_str(ticker);
    match redis_hash::hget(conn, "user_open_order_sums", field.as_str()) {
        Ok(sums) => {
            let sums: String = sums;
            sums
        },
        Err(e) => String::from("0")
    }
}


pub fn get_sum_of_orders_for_price_point(conn: &mut Connection, order_type: &OrderType, pair: &String, price: u128) -> String {
    let mut field: String = String::from(pair);
    field.push('-');
    field.push_str(price.to_string().as_str());

    let sums_field = match order_type {
      OrderType::BID => "bids-sums",
      OrderType::ASK => "asks-sums",
      OrderType::DELETE => panic!("Can't get sum of delete orders")
    };


    match redis_hash::hget(conn, sums_field, field.as_str()) {
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

    let mut all_orders: Vec<Order> = Vec::new();

    let mut order_amount_count: u128 = 0;

    let mut existing_orders = match &order.order_type {
        OrderType::ASK => {
            get_orders_by_price_index(conn, &OrderType::BID, &order.pair, -1, 0)
        },
        OrderType::BID => {
            get_orders_by_price_index(conn, &OrderType::ASK, &order.pair, 0, -1)
        },
        _ => panic!("Cannot get Delete type orders from orderbook")
    };

    if existing_orders.is_empty() {
        return all_orders;
    }

    for mut ord in existing_orders {
        order_amount_count += ord.amount;

        all_orders.push(ord);
        if order_amount_count >= order.amount {
            break;
        }
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
