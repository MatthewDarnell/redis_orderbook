extern crate redis;
use redis::{connection, types::redis_key, types::redis_hash};
extern crate order;
extern crate uuid;
extern crate trade;
use uuid::Uuid;

use order::order::{store, order_type, order_execution_type};
use order::order::order_pair::Pair;
use order::order::orderbook;

use trade::trade::place_trade;
use order::order::store::order_pair_store::add_new_pair;

fn main() {

}
