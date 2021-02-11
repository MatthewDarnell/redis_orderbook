# Redis Orderbook

A fast in memory orderbook written in Rust using Redis Data Structures


### What is this?
This is an implementation of an orderbook written in Rust and running in Redis.
It supports `LIMIT` and `MARKET` orders, as well as `fill or kill` orders. It also supports pairs of any type.

### How can I use this?
This is intended to be easy to plug-and-play with any existing service that tracks user accounts and balances. Build the library and integrate with your application, or build the binaries and interact with redis pubsub and a simple http api.

### How to build?

First, install rust. 

`Windows: https://www.rust-lang.org/tools/install`

`Linux curl https://sh.rustup.rs -sSf | sh`

Then build with `cargo`:

`cargo build --bins`

Current binaries produced in `target` output folder:

```asm
redis_pubsub_listener
redis_created_orders_listener
redis_trades_completed_listener
http_api
```

You must also have `redis-server` running to run the application. `https://redis.io/download`

### Example:

#### *1. Start Redis*
    redis-server

#### *2. Run the http api*
    cargo run --bin http_api

This will start the server on `127.0.0.1:3000` You can see the available API if you navigate your web browser there. For now, create a new trading pair:



```commandline
$ curl -H "Content-type: application/json" "http://127.0.0.1:3000/add_pair?price_ticker=btc&ref_ticker=usd"
{"price_ticker":"btc","ref_ticker":"usd","uuid":"4ef93380-e3eb-40a3-a3d5-2cee1bf9d201"}
```

This call responds with a uuid, which is the newly created`pair_id`. You should store this id in your database.

Now let's take a look at the existing orderbook for this pair and make sure it is empty:

```commandline
$ curl http://127.0.0.1:3000/orderbook/4ef93380-e3eb-40a3-a3d5-2cee1bf9d201
{"asks": [], "bids": []}
```

#### *3. Run the pubsub listener*
    cargo run --bin redis_pubsub_listener

Now let's submit an order to the orderbook for this pair using `redis-cli`. In your application would just use the redis library to `publish` orders to the channel.

```
redis-cli publish incoming_orders "{ \"user_id\": \"user_id_1\", \"order_type\": \"BID\", \"order_execution_type\":\"LIMIT\", \"fill_or_kill\": false, \"price\": 100, \"amount\": 1, \"pair\": \"4ef93380-e3eb-40a3-a3d5-2cee1bf9d201\" }"
(integer) 1
```

Now let's look at the orderbook:

```commandline
$ curl http://127.0.0.1:3000/orderbook/4ef93380-e3eb-40a3-a3d5-2cee1bf9d201
{"asks": [], "bids": [{"price": "100", "sum": "1"}]}
```

We placed an order! Let's fill it with an `ASK` that will leave some leftover in the orderbook:

```
redis-cli publish incoming_orders "{ \"user_id\": \"user_id_2\", \"order_type\": \"ASK\", \"order_execution_type\":\"LIMIT\", \"fill_or_kill\": false, \"price\": 99, \"amount\": 2, \"pair\": \"4ef93380-e3eb-40a3-a3d5-2cee1bf9d201\" }"
(integer) 1
```

And the resulting orderbook:

```commandline
$ curl http://127.0.0.1:3000/orderbook/4ef93380-e3eb-40a3-a3d5-2cee1bf9d201
{"asks": [{"price": 99, "sum": 1}], "bids": []}
```

Now let's check this users' open orders and balance.

```commandline
$ curl http://127.0.0.1:3000/user_order_sums/user_id_2/btc
1

$ curl http://127.0.0.1:3000/user_order_sums/user_id_2/usd
0
```

This user has an `ASK` for 1 btc, so his _open_ orderbook balance of btc is 1, you should subtract that from the users' withdrawable balance in your application. Conversely, when a user places a `BID`, his open balance of the `ref_ticker`, in this case `USD` in this case would be the `price * amount` (A bid of 2 btc at 40000 usd would require 80000 usd)


Some notes: A `MARKET` order currently does not leave a resulting order in the orderbook, it places an order at any price and does not place a new order for any leftover amount.


### Other features
#### 1. To keep track of placed orders, redis `subscribe` to the channel `created_orders`

You can run the `redis_created_orders_listener` as a demo:

    cargo run --bin redis_created_orders_listener

    "{\"user_id\": \"user_id_1\", \"uuid\": \"b4b09e71-0813-4d8c-a11b-fb33be73d8f0\"}"


#### 2. To keep track of completed trades, redis `subscribe` to the channel `trades_completed`

You can run the `redis_trades_completed_listener` as a demo:

    cargo run --bin redis_trades_completed_listener

    "{\"pair_id\":\"4ef93380-e3eb-40a3-a3d5-2cee1bf9d201\",\"pair\":\"{\\\"price_ticker\\\":\\\"btc\\\",\\\"ref_ticker\\\":\\\"usd\\\",\\\"uuid\\\":\\\"4ef93380-e3eb-40a3-a3d5-2cee1bf9d201\\\"}\",\"execution_price\":\"100\",\"filled_amount\":\"1\",\"side\":\"ASK\",\"bid_user_id\":\"user_id_1\",\"ask_user_id\":\"user_id_2\",\"bid_order_id\":\"b4b09e71-0813-4d8c-a11b-fb33be73d8f0\",\"ask_order_id\":\"d444bb31-4d90-4b66-a4c4-a8a989886799\",\"timestamp\":\"1613077532\"}"



#### 3. To Retrieve a users' open orders, request the appropriate api from the http server:

```commandline
$ curl http://127.0.0.1:3000/user_open_orders/user_id_1/4ef93380-e3eb-40a3-a3d5-2cee1bf9d201
[
  {
    "uuid": "6b092b56-67c6-4019-bdd0-2c1d368b838b",
    "user_id": "user_id_1",
    "order_type": "BID",
    "order_execution_type": "LIMIT",
    "fill_or_kill": false,
    "price": 100,
    "amount": 1,
    "pair": {
      "price_ticker": "btc",
      "ref_ticker": "usd",
      "uuid": "4ef93380-e3eb-40a3-a3d5-2cee1bf9d201"
    },
    "timestamp": 1613077682
  }
]
```

#### 4. To Delete an Order, redis `publish` a json string containing the `order_id` and type `DELETE`


```commandline
redis-cli publish incoming_orders "{\"order_type\": \"DELETE\", \"uuid\": \"6b092b56-67c6-4019-bdd0-2c1d368b838b\"}"
(integer) 1
```


### How is this orderbook designed?

This orderbook uses provided Redis data structures. Orders themselves are serialized and placed into a hash table, while their uuids are placed into a set. The orderbook itself is a redis sorted set, with each score representing the price and each value the key of a fifo list containing the user id and order uuid.

Operations:

    Insertion: O(1)
    Deletion: O(n), where n is the number of existing orders at the same price
    Lookup: O(1)



### Final Notes

This orderbook is limited to 64 bit integers (a constraint of redis' sorted sets). You must multiply any amount by some integer which guarantees the passed amount is a valid int64.

If dealing with cryptocurrencies, you may need to truncate at some number of decimal places. i.e. MAX Bitcoin is `21000000.00000000`, and int64 max is `9223372036854775807`, so this library should be able to handle the full max of `21000000.00000000 * 100000000 = 2100000000000000`


However, Ethereum maintains 12 decimal places 

`1000000.000000000001 * 1000000000000 = 1000000000000000001` which is greater than MAX int64, so you would have to truncate the maximum number of decimal places handled.


### Disclaimer

This orderbook is a work in progress and comes with no real or implied warranty. If you find any bugs, please don't hesitate to submit a fix!
