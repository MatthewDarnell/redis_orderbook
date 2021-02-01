use std::env;
use warp::Filter;
extern crate uuid;
use order;
use order::order::order_pair::Pair;
use std::collections::HashMap;
extern crate redis;
use redis::connection;
use order::order::orderbook::get_user_open_order_sum;
use std::net::{Ipv4Addr, IpAddr};
use std::str::FromStr;

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

    let user_open_orders = warp::path!("user_open_orders" / String / String).map(|user_id: String, pair_id: String| {
        match uuid::Uuid::parse_str(&pair_id) {
            Ok(uuid) => {
                match connection::get_connection(None) {
                    Ok(mut conn) => {
                        match order::order::store::order_pair_store::get_pair_by_id(&mut conn, uuid) {
                            Some(pair) => {
                                match order::order::store::order_store::get_orders_by_user_id_and_pair_id(&mut conn, user_id.as_str(), pair_id.as_str()) {
                                    Some(order_list) => {
                                        match serde_json::to_string_pretty(&order_list) {
                                            Ok(result) => format!("{}", result.as_str()),
                                            Err(e) => format!("{}", e)
                                        }
                                    },
                                    None => format!("[]")
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

    let home = warp::get().map(|| format!("API:\n\
    /all_pairs\n\
    /add_pair?price_ticker=<price_ticker>&ref_ticker=<ref_ticker>\n\
    /user_open_orders/<user_id>/<pair_id>\n\
    /orderbook/<pair_id>\n\
    /user_order_sums/<user_id>/<ticker>"
    ));


    let routes = warp::get().and(pairs.or(orderbook).or(user_open_orders).or(add_pair).or(order_sums).or(home));


    let mut host = String::from("127.0.0.1");
    let mut port: u16 = 3000;
    match env::var("REDIS_ORDERBOOK_HTTP_API_HOST") {
        Ok(hostname) => {
            host = hostname;
        },
        Err(_) => {
        }
    }

    match env::var("REDIS_ORDERBOOK_HTTP_API_PORT") {
        Ok(env_var_port) => {
            port = env_var_port.parse().unwrap();
        },
        Err(_) => {
        }
    }

    println!("Redis Orderbook HTTP API Server listening at: <{}:{}>", host.as_str(), port);
    let address: IpAddr = IpAddr::from_str(host.as_str()).expect("Unable to parse as Ip Address");
    warp::serve(routes)
        .run((address, port))
        .await;
}