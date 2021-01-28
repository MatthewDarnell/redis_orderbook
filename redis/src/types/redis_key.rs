use redis::Commands;
pub use redis::FromRedisValue;
use redis::ToRedisArgs;
pub use redis::RedisResult;

pub fn get<'a, T: FromRedisValue> (conn: &'a mut redis::Connection, key: &'a str) -> redis::RedisResult<T> {
    conn.get(key)
}

pub fn set<T: ToRedisArgs> (conn: &mut redis::Connection, key: &str, val: T) -> redis::RedisResult<()> {
    conn.set(key, val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection;
    #[test]
    fn test_get_set() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_key_test_key";
            let value = 5;
            self::set(&mut conn, key, value).expect("Error setting value");
            let result: i32 = self::get(&mut conn, key).unwrap();
            connection::del(&mut conn, key).expect("Error deleting value");
            assert_eq!(result, value);
        } else {
            assert_eq!(1, 2);
        }
    }
}