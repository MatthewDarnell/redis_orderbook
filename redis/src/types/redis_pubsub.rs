use redis::Commands;
pub use redis::FromRedisValue;
pub use redis::RedisResult;

pub fn publish<'a, 'b> (conn: &'a mut redis::Connection, key: &'a str, field: &'b str) -> redis::RedisResult<()> {
    conn.publish(key, field)
}
pub fn subscribe(conn: &mut redis::PubSub, channel: &str) {
    conn.subscribe(channel);
}
pub fn unsubscribe(conn: &mut redis::PubSub, channel: &str) {
    conn.unsubscribe(channel);
}

pub fn get_message(conn: &mut redis::PubSub) -> RedisResult<String> {
    let msg = conn.get_message()?;
    msg.get_payload()
}