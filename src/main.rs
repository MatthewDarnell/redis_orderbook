extern crate redis;
use redis::{connection, types::redis_key, types::redis_hash};
extern crate order;
extern crate uuid;
use uuid::Uuid;

use order::order::{store, order_type, order_execution_type};
use order::order::order_pair::Pair;
use order::order::orderbook;
fn main() {

    let result = connection::get_connection(None);
    match result {
        Ok(mut c) => {

            let pair = &Pair::new("btc", "usd", uuid::Uuid::new_v4());

            let o: order::order::Order = order::order::Order::new("test_user_id", order_type::OrderType::BID, order_execution_type::OrderExecutionType::LIMIT, true, 1800, 1001, pair);
            let oa: order::order::Order = order::order::Order::new("test_user_id", order_type::OrderType::ASK, order_execution_type::OrderExecutionType::LIMIT, true, 1000, 1001, pair);


            let ob: order::order::Order = order::order::Order::new("test_user_id2", order_type::OrderType::BID, order_execution_type::OrderExecutionType::LIMIT, true, 900, 1001, pair);
            let oc: order::order::Order = order::order::Order::new("test_user_id2", order_type::OrderType::ASK, order_execution_type::OrderExecutionType::MARKET, true, 905, 1002, pair);



            let od: order::order::Order = order::order::Order::new("test_user_id3", order_type::OrderType::BID, order_execution_type::OrderExecutionType::LIMIT, true, 1000, 1001, pair);
            store::order_store::create_order(&mut c, &o);
            store::order_store::create_order(&mut c, &oa);
            store::order_store::create_order(&mut c, &ob);
            store::order_store::create_order(&mut c, &oc);
            store::order_store::create_order(&mut c, &od);
            store::order_execution_type_store::get_orders_by_price(&mut c, &order_type::OrderType::ASK, pair, 0, 1000000000);

            let v = orderbook::get_matching_market_orders_for_execution(&mut c, &oc);
            for f in &v {
                println!("retrieved: {}", f.serialize());
            }
        },
        Err(e) => panic!("Error Getting Redis Connection: <{}>", e)
    }
}
