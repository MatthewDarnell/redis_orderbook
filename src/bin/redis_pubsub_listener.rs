extern crate rust_redis;
extern crate redis;
pub use redis::connection::{ Connection, PubSub };

fn main() {
    rust_redis::redis_pubsub::init_listening_for_orders("incoming_orders", "created_orders");
}