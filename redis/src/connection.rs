extern crate redis;
use redis::Commands;
pub use redis::FromRedisValue;
pub use self::redis::RedisResult;
pub use redis::ToRedisArgs;
pub use redis::Connection;
const REDIS_IP: &str = "redis://127.0.0.1:6379";

//@ip_addr: Pass <None> to use default <redis://127.0.0.1:6379>
pub fn get_connection(ip_addr: Option<&str>) -> Result<redis::Connection, redis::RedisError> {
    let client;
    if let Some(ip) = ip_addr {
        println!("Opening Redis Connection at: <{}>", ip);
        client = redis::Client::open(ip)?;
    } else {
        println!("Opening Redis Connection at default URL: <{}>", REDIS_IP);
        client = redis::Client::open(REDIS_IP)?;
    }
    client.get_connection()
}

pub fn del(conn: &mut redis::Connection, key: &str) -> RedisResult<()> {
    conn.del(key)
}