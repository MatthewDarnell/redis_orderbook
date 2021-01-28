use redis::Commands;
pub use redis::FromRedisValue;
use redis::ToRedisArgs;
pub use redis::RedisResult;

pub fn sadd<'a> (conn: &'a mut redis::Connection, key: &'a str, value: &'a str) -> redis::RedisResult<()> {
    conn.sadd(key, value)
}

pub fn sismember<U: ToRedisArgs, V: ToRedisArgs> (conn: &mut redis::Connection, key: U, member: V) -> redis::RedisResult<bool> {
    conn.sismember(key, member)
}

pub fn smembers<T: FromRedisValue> (conn: &mut redis::Connection, key: &str) -> redis::RedisResult<Vec<T>> {
    conn.smembers(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection;
    #[test]
    fn test_sadd_smembers() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_set_test_key";

            let mut values = vec!["adam".to_string(), "john".to_string(), "george".to_string()];
            values.sort();
            for v in &values {
                self::sadd(&mut conn, key, v).expect("Error adding member to set");
            }
            match self::smembers(&mut conn, key) {
                Ok(res) => {
                    let mut res: Vec<String> = res;
                    res.sort();
                    let matching = res.iter().zip(values.iter()).filter(|&(a, b)| a == b).count();
                    assert_eq!(values.len(), matching);
                },
                Err(e) => {
                    println!("Error retrieving smembers: {}", e);
                    assert_eq!(1, 2);
                }
            }
            connection::del(&mut conn, key).expect("Error deleting value");
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_sadd_sismember() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let key = "redis_set_test_key_2";

            let mut values = vec!["adam".to_string(), "john".to_string(), "george".to_string()];
            values.sort();
            for v in &values {
                self::sadd(&mut conn, key, v).expect("Error adding member to set");
            }

            match self::sismember(&mut conn, key, "john") {
                Ok(res) => {
                    assert_eq!(res, true);
                },
                Err(e) => {
                    println!("Error retrieving smembers: {}", e);
                    assert_eq!(1, 2);
                }
            }
            connection::del(&mut conn, key).expect("Error deleting value");
        } else {
            assert_eq!(1, 2);
        }
    }

}