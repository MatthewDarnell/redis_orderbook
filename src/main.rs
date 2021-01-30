extern crate redis;
use redis::{connection, types::redis_key, types::redis_hash};
extern crate order;
extern crate uuid;
extern crate trade;
use uuid::Uuid;

use std::io;

use order::order::{store, order_type, order_execution_type};
use order::order::order_pair::Pair;
use order::order::orderbook;

use trade::trade::place_trade;
use order::order::store::order_pair_store::add_new_pair;
use std::process::exit;

mod lib;

fn print_menu() {
    println!("\n\tMenu:\n1. Connect to Redis Ip Address <defaults to 127.0.0.1:6379>\n2. Place a Trade\n3. Get Current Orderbook\n4. Get All Pairs\n5. Create a new Pair\n6. Quit");
}

fn get_input_string() -> String {
    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    choice.trim().to_string()
}

fn get_input(min: u32, max: u32) -> u32 {
    loop {
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        let choice: u32 = match choice.trim().parse() {
            Ok(num) => { num },
            Err(_) => {
                println!("Invalid Choice! Please Try Again");
                continue
            },
        };
        if choice < min || choice > max {
            println!("Invalid Choice! Please Try Again");
            continue;
        }
        return choice;
    }
}

fn main() {
    let mut redis_ip: String = "redis://127.0.0.1:6379".to_string();
    let mut connection = lib::init(redis_ip.as_str()).unwrap();
    println!("Welcome to Redis Orderbook CLI.\t{}", &redis_ip);
    print_menu();

    loop {

        match get_input(1, 6) {
            1 => {
                println!("Enter Ip: ");
                let ip = get_input_string();
                redis_ip = ip;
                print!("{}[2J", 27 as char);
            },
            2 => {
                println!("Enter user_id:");
                let user_id = get_input_string();
                println!("Want to publish completed trade to redis pubsub (channel=trades_completed) [true/false]:");
                let publish: bool = get_input_string().parse().unwrap_or_else(|_| true);

                println!("Enter order type (BID/ASK):");
                let mut order_type = get_input_string().to_ascii_uppercase();
                while order_type != "BID".to_string() && order_type != "ASK".to_string() {
                    println!("Invalid Order Type, try again: ");
                    order_type = get_input_string().to_ascii_uppercase();
                }

                println!("Enter order execution type (LIMIT/MARKET):");
                let mut order_execution_type = get_input_string().to_ascii_uppercase();
                while order_execution_type != "LIMIT".to_string() && order_execution_type != "MARKET".to_string() {
                    println!("Invalid Order Type, try again: ");
                    order_execution_type = get_input_string().to_ascii_uppercase();
                }

                println!("Enter fill or kill [true/false]:");
                let fill_or_kill: bool = get_input_string().parse().unwrap_or_else(|_| false);

                println!("Enter price as integer:");
                let price: u128 = get_input_string().parse().unwrap_or_else(|_| 0);

                println!("Enter amount as integer:");
                let amount: u128 = get_input_string().parse().unwrap_or_else(|_| 0);

                println!("Enter pair id:");
                let pair_id = get_input_string();

                match lib::place_trade(
                    &mut connection,
                    publish,
                    user_id.as_str(),
                    order_type.as_str(),
                    order_execution_type.as_str(),
                    fill_or_kill,
                    price,
                    amount,
                    pair_id.as_str()
                ) {
                    Ok(res) => println!("{}", res),
                    Err(e) => println!("{}", e),
                }

            },
            3 => {
                println!("Input Pair Id: ");
                let pair_id = get_input_string();
                match lib::get_orderbook(&mut connection, pair_id.as_str(), 32) {
                    Ok(orderbook) => println!("{}", orderbook),
                    Err(_) => {}
                }
            },
            4 => {  //get all pairs
                match lib::get_all_pairs(&mut connection) {
                    Ok(res) => {
                        if res.len() > 1 {
                            println!("{}", res);
                        } else {
                            println!("[]");
                        }
                    },
                    Err(_) => println!("Failed to Fetch Pairs")
                }
            },
            5 => {  //add new pair
                println!("Add a new pair (Price_Ticker/Ref_Ticker):\nEnter Price Ticker:");
                let price_ticker = get_input_string();
                println!("Enter Ref Ticker:");
                let ref_ticker = get_input_string();
                lib::create_pair(&mut connection, price_ticker.as_str(), ref_ticker.as_str());
                println!("ok");
            },
            6 => {
                println!("Goodbye!");
                exit(0);
            },
            _ => {}
        }
    }





}
