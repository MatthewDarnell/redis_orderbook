use redis::Commands;
pub use redis::FromRedisValue;
use redis::ToRedisArgs;
pub use redis::RedisResult;

pub fn zadd<T: ToRedisArgs, U: ToRedisArgs>(conn: &mut redis::Connection, key: &str, value: T, score: U) -> redis::RedisResult<()> {
    conn.zadd(key, value, score)
}

pub fn zrange<T: ToRedisArgs, RV: FromRedisValue> (conn: &mut redis::Connection, key: T, start: isize, stop: isize) -> redis::RedisResult<Vec<RV>> {
    conn.zrange(key, start, stop)
}

pub fn zrem<T: ToRedisArgs>(conn: &mut redis::Connection, key: &str, value: T) -> redis::RedisResult<()> {
    conn.zrem(key, value)
}

pub fn zrevrange<T: ToRedisArgs, RV: FromRedisValue> (conn: &mut redis::Connection, key: T, stop: isize, start: isize) -> redis::RedisResult<Vec<RV>> {
    conn.zrevrange(key, stop, start)
}

pub fn zrangebyscore<T: ToRedisArgs, U: ToRedisArgs, V: ToRedisArgs, RV: FromRedisValue> (conn: &mut redis::Connection, key: T, min: U, max: V) -> redis::RedisResult<Vec<RV>> {
    conn.zrangebyscore(key, min, max)
}

pub fn zrevrangebyscore<T: ToRedisArgs, U: ToRedisArgs, V: ToRedisArgs, RV: FromRedisValue> (conn: &mut redis::Connection, key: T, max: U, min: V) -> redis::RedisResult<Vec<RV>> {
    conn.zrevrangebyscore(key, max, min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection;
    #[test]
    fn test_zadd_zrange() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_sorted_set_test_key";
            let value = "value";
            let score = 10;
            self::zadd(&mut conn, key, value, score).expect("Error setting value");
            match self::zrange(&mut conn, key, 0, 10) {
                Ok(res) => {
                    let res: Vec<String> = res;
                    assert_eq!(res.len(), 1);
                    let r = &res[0];
                    assert_eq!(r, &"value".to_string());
                },
                Err(e) => {
                    println!("Error getting zrange: {}", e);
                    assert_eq!(1, 2);
                }
            }
            connection::del(&mut conn, key).expect("Error deleting value");
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_zrangebyscore() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_sorted_set_test_key_2";
            let value = vec![("value1".to_string(), 18), ("value2".to_string(), 15), ("value3".to_string(), 6)];
            let score = 10;

            for (v, s) in &value {
                self::zadd(&mut conn, key, v, *s).expect("Error setting value");
            }

            match self::zrangebyscore(&mut conn, key, 7, 20) {
                Ok(res) => {
                    let res: Vec<String> = res;
                    assert_eq!(res.len(), 2);
                    let r = &res[0];
                    assert_eq!(r, &"value2".to_string());
                },
                Err(e) => {
                    println!("Error getting zrange: {}", e);
                    assert_eq!(1, 2);
                }
            }
            connection::del(&mut conn, key).expect("Error deleting value");
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_zrevrangebyscore() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_sorted_set_test_key_3";
            let value = vec![("value1".to_string(), 18), ("value2".to_string(), 15), ("value3".to_string(), 6)];
            let score = 10;

            for (v, s) in &value {
                self::zadd(&mut conn, key, v, *s).expect("Error setting value");
            }

            match self::zrevrangebyscore(&mut conn, key, 20, 7) {
                Ok(res) => {
                    let res: Vec<String> = res;
                    assert_eq!(res.len(), 2);
                    let r = &res[0];
                    assert_eq!(r, &"value1".to_string());
                },
                Err(e) => {
                    println!("Error getting zrange: {}", e);
                    assert_eq!(1, 2);
                }
            }
            connection::del(&mut conn, key).expect("Error deleting value");
        } else {
            assert_eq!(1, 2);
        }
    }
}