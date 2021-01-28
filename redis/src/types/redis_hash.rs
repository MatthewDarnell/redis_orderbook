use redis::Commands;
pub use redis::FromRedisValue;
use redis::ToRedisArgs;
pub use redis::RedisResult;

pub fn hget<'a, 'b, RV: FromRedisValue> (conn: &'a mut redis::Connection, key: &'a str, field: &'b str) -> redis::RedisResult<RV> {
    conn.hget(key, field)
}

pub fn hmget<T: ToRedisArgs, RV: FromRedisValue> (conn: &mut redis::Connection, key: &str, field:  T) -> redis::RedisResult<RV> {
    conn.hget(key, field)
}

pub fn hincrby<T: ToRedisArgs> (conn: &mut redis::Connection, key: &str, field: &str, delta: T) -> redis::RedisResult<()> {
    conn.hincr(key, field, delta)
}

pub fn hset<F: ToRedisArgs, V: ToRedisArgs> (conn: &mut redis::Connection, key: &str, field: F, val: V) -> redis::RedisResult<()> {
    conn.hset(key, field, val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection;
    #[test]
    fn test_hget_hset() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_hash_test_key";
            let field = "field";
            let value = 5;
            self::hset(&mut conn, key, field, value).expect("Error h-setting value");
            let result: i32 = self::hget(&mut conn, key, field).unwrap();
            connection::del(&mut conn, key).expect("Error deleting value");
            assert_eq!(result, value);
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_hmget() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_hash_test_key_2";
            let values = vec![("field1".to_string(), "one".to_string()), ("field2".to_string(), "two".to_string()), ("field3".to_string(), "three".to_string())];
            for (f, v) in &values {
                self::hset(&mut conn, key, f, v).expect("Error h-setting value");
            }
            let fields: Vec<String> = vec!["field1".to_string(), "field2".to_string(), "field3".to_string()];
            let values: Vec<String> = vec!["one".to_string(), "two".to_string(), "three".to_string()];

            let result: Vec<String> = self::hmget(&mut conn, key, fields).unwrap();
            let matching = result.iter().zip(values.iter()).filter(|&(a, b)| a == b).count();
            connection::del(&mut conn, key).expect("Error deleting value");
            assert_eq!(matching, 3);
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_hget_hincrby() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_hash_test_key_3";
            let field = "field";
            self::hset(&mut conn, key, field, 0).expect("Error h-setting value");
            let value = 5;
            self::hincrby(&mut conn, key, field, value).expect("Error h-incrby value");
            let result: i32 = self::hget(&mut conn, key, field).unwrap();
            connection::del(&mut conn, key).expect("Error deleting value");
            assert_eq!(result, value);
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_hget_hset_str() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_hash_test_key_4";
            let field = "field";
            let value = "value";
            self::hset(&mut conn, key, field, value).expect("Error h-setting value");
            let result: String = self::hget(&mut conn, key, field).unwrap();
            connection::del(&mut conn, key).expect("Error deleting value");
            assert_eq!(result, value);
        } else {
            assert_eq!(1, 2);
        }
    }
}