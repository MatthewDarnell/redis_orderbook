use redis::connection::Connection;
use crate::order::orderbook::get_sum_of_orders_for_price_point;
use crate::order::order_type::OrderType;

pub mod order;

pub fn get_orderbook(conn: &mut Connection, pair_id: String, depth: u32) -> Result<String, String> {
    let pair_id = match uuid::Uuid::parse_str(pair_id.as_str()) {
        Ok(v) => v,
        Err(_) => return Err(String::from("Invalid uuid"))
    };
    let pair = match order::store::order_pair_store::get_pair_by_id(conn, pair_id) {
        Some(pair) => pair,
        None => return Err(String::from("Invalid pair uuid"))
    };

    let mut ask_key = String::from("asks-");
    ask_key.push_str(&pair.uuid);

    let order_asks_q: Vec<(String, String)> = match order::store::order_type_store::get_sorted_set_for_order_type(conn, ask_key.as_str(), -1, false) {
        Ok(v) => {
            let mut ret_val: Vec<(String, String)> = Vec::new();
            for list in &v {
                let pair_id = String::from(&list[5..41]);
                let price = String::from(&list[42..]);
                ret_val.push((pair_id, price));
            }
            ret_val
        },
        Err(_) => Vec::new()
    };



    let mut ask_key = String::from("bids-");
    ask_key.push_str(&pair.uuid);

    let order_bids_q = match order::store::order_type_store::get_sorted_set_for_order_type(conn, ask_key.as_str(), -1, true) {
        Ok(v) => {
            let mut ret_val: Vec<(String, String)> = Vec::new();
            for list in &v {
                let pair_id = String::from(&list[5..41]);
                let price = String::from(&list[42..]);
                ret_val.push((pair_id, price));
            }
            ret_val
        },
        Err(_) => Vec::new()
    };

    let mut json = String::from("{\"asks\": [");

    if order_asks_q.len() < 1 {
        json.push_str("], \"bids\": [");
    } else {
        for (pair_id, price) in &order_asks_q {
            let p: u128 = price.parse().unwrap();
            let sum = get_sum_of_orders_for_price_point(conn, &OrderType::ASK, &pair_id, p);
            if sum == "0" {
                continue;
            }
            json.push_str("{\"price\": ");
            json.push_str(price.as_str());
            json.push_str(", \"sum\": ");
            json.push_str(sum.as_str());
            json.push_str("}, ");
        }
        if order_asks_q.len() > 0 {
            json.pop();
            json.pop();
        }

        json.push_str("], \"bids\": [");
    }


    if order_bids_q.len() < 1 {
        json.push_str("]}");
    } else {
        for (pair_id, price) in &order_bids_q {

            let p: u128 = price.parse().unwrap();
            let sum = get_sum_of_orders_for_price_point(conn, &OrderType::BID, &pair_id, p);

            if sum == "0" {
                continue;
            }
            json.push_str("{\"price\": \"");
            json.push_str(price.as_str());
            json.push_str("\", \"sum\": \"");
            json.push_str(sum.as_str());
            json.push_str("\"}, ");
        }

        if order_bids_q.len() > 0 {
            json.pop();
            json.pop();
        }

        json.push_str("]}");
    }




    Ok(json)
}