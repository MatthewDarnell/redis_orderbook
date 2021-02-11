extern crate redis;
use redis::connection;
use redis::connection::get_connection;
use serde_json;

fn main() {
    println!("Listening for created orders on channel <created_orders>: Ctrl^c to quit");
    let mut conn = get_connection(None).unwrap();
    let mut conn = conn.as_pubsub();
    conn.subscribe("created_orders");
    loop {
        let payload: String = conn.get_message().unwrap().get_payload().unwrap();
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
}