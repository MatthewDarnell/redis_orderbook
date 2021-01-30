use warp::Filter;
extern crate uuid;
use order;
use order::order::order_pair::Pair;
use std::collections::HashMap;
extern crate redis;
use redis::connection;
use order::order::orderbook::get_user_open_order_sum;

#[tokio::main]
async fn main() {

    let pairs = warp::path("all_pairs").map(|| {
        match connection::get_connection(None) {
            Ok(mut conn) => {
                let pairs = order::order::store::order_pair_store::get_pairs(&mut conn);
                let pairs: Vec<Pair> = pairs;
                let response = serde_json::to_string(&pairs).unwrap();
                format!("{}", response)
            },
            Err(_) => format!("Unable to access redis server. Is it running?")
        }
    });

    let orderbook = warp::path!("orderbook" / String).map(|pair_id: String| {
        match uuid::Uuid::parse_str(&pair_id) {
            Ok(uuid) => {
                match connection::get_connection(None) {
                    Ok(mut conn) => {
                        match order::order::store::order_pair_store::get_pair_by_id(&mut conn, uuid) {
                            Some(pair) => {
                                match order::get_orderbook(&mut conn, pair.uuid, 32) {
                                    Ok(orderbook) => {
                                        format!("{}", orderbook)
                                    },
                                    Err(e) => format!("Unable to retrieve orderbook: {}", e)
                                }
                            },
                            None => format!("{} is not a known pair", pair_id)
                        }
                    },
                    Err(_) => format!("Unable to access redis server. Is it running?")
                }
            },
            Err(_) => format!("Invalid Pair Uuid {}", pair_id)
        }
    });



    let order_sums = warp::path!("user_order_sums" / String / String).map(|user_id: String, ticker: String| {
        match connection::get_connection(None) {
            Ok(mut conn) => {
                let sum: u128 = get_user_open_order_sum(&mut conn, user_id.as_str(), ticker.as_str()).parse().unwrap();
                format!("{}", sum)
            },
            Err(_) => format!("Unable to access redis server. Is it running?")
        }
    });


    // get /example1?key=value
    // demonstrates an optional parameter.
    let add_pair = warp::get()
        .and(warp::path("add_pair"))
        .and(warp::query::<HashMap<String, String>>())
        .map(|p: HashMap<String, String>| {
            match p.get("price_ticker") {
                Some(price_ticker) => {
                    match p.get("ref_ticker") {
                        Some(ref_ticker) => {
                            match connection::get_connection(None) {
                                Ok(mut conn) => {
                                    let existing_pairs = order::order::store::order_pair_store::get_pairs(&mut conn);
                                    for pair in &existing_pairs {
                                        let existing_price_ticker = &pair.price_ticker;
                                        let existing_ref_ticker = &pair.ref_ticker;
                                        if price_ticker.trim() == existing_price_ticker.trim() && ref_ticker.trim() == existing_ref_ticker.trim() {
                                            return format!("Existing {} / {} pair exists. Uuid: {}", price_ticker, ref_ticker, pair.uuid);
                                        }
                                    }
                                    let pair_id = uuid::Uuid::new_v4();
                                    let pair: Pair = Pair::new(price_ticker.as_str(), ref_ticker.as_str(), pair_id);
                                    order::order::store::order_pair_store::add_new_pair(&mut conn, &pair);
                                    format!("{}", pair.serialize())
                                },
                                Err(_) => format!("Unable to access redis server. Is it running?")
                            }
                        },
                        None => format!("No ref_ticker found in querystring")
                    }
                },
                None => format!("No price_ticker found in querystring"),
            }
        }
        );


    let routes = warp::get().and(pairs.or(orderbook).or(add_pair).or(order_sums));

    println!("Redis Orderbook HTTP Server listening on port 3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}