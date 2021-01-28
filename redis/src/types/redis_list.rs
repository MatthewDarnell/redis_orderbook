use redis::Commands;
pub use redis::FromRedisValue;
use redis::ToRedisArgs;
pub use redis::RedisResult;


pub fn llen<'a> (conn: &'a mut redis::Connection, list: &'a str) -> redis::RedisResult<i32> {
    conn.llen(list)
}

pub fn lrange<T: FromRedisValue> (conn: &mut redis::Connection, list: &str, start: i32, stop: i32) -> redis::RedisResult<Vec<T>> {
    conn.lrange(list, start as isize, stop as isize)
}

pub fn rpush<T: ToRedisArgs> (conn: &mut redis::Connection, list: &str, val: T) -> redis::RedisResult<()> {
    conn.rpush(list, val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection;

    #[test]
    fn test_list_length() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let list = "redis_list_test_key";
            let len = self::llen(&mut conn, list).expect("Error retrieving list length");
            assert_eq!(len, 0);

            let values = vec!["one".to_string(), "two".to_string(), "three".to_string()];
            for v in &values {
                self::rpush(&mut conn, list, v).expect("Error rpush'ing list values");
            }


            let len = self::llen(&mut conn, list).expect("Error retrieving list length");
            connection::del(&mut conn, list).expect("Error deleting value");
            assert_eq!(len, 3);
        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_list_retrieval() {
        if let Ok(mut conn) = connection::get_connection(None) {
            let list = "redis_list_retrieval_test_key";
            let len = self::llen(&mut conn, list).expect("Error retrieving list length");
            assert_eq!(len, 0);

            let values = vec!["one".to_string(), "two".to_string(), "three".to_string()];
            for v in &values {
                self::rpush(&mut conn, list, v).expect("Error rpush'ing list values");
            }

            match self::lrange(&mut conn, list, 0, 3) {
                Ok(res) => {
                    let res: Vec<String> = res;
                    let matching = res.iter().zip(values.iter()).filter(|&(a, b)| a == b).count();
                    assert_eq!(res.len(), matching);
                },
                Err(e) => {
                    println!("Error reading lrange on {} : {}", list, e);
                    assert_eq!(1, 2);
                }
            }
            connection::del(&mut conn, list).expect("Error deleting value");
        } else {
            assert_eq!(1, 2);
        }
    }
}