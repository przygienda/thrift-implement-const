//! implements redis [https://redis.io/] persistency traits for the Thrift models under Rust
//!
use std::fmt::Display;
use std::hash::Hash;
use std::collections::BTreeSet;
use std::collections::BTreeMap;

use redis;
use protocol;

use redis::ToRedisArgs;
use redis::Commands;
use redis::PipelineCommands;

/// trait provides persistency for `thrift::protocol::ThriftTyped` on
/// [https://redis.io/]
pub trait RedisPersistency<K>
: protocol::ThriftTyped + redis::FromRedisValue + Sized
where K: redis::ToRedisArgs + Clone + Display
{
	fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()>;
	fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<Self>>;
}

impl<K, X> RedisPersistency<K>
for Vec<X>
where X: Default + protocol::ThriftTyped +
redis::ToRedisArgs + redis::FromRedisValue + Sized,
	  K: redis::ToRedisArgs + Clone + Display {
	fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()>
	{
		// try!(redis::cmd("SET").arg("k1").arg(&[5,6]).execute(conn));
		// println!("writing {:?} {:?} {:?}", key.clone(), self, self.to_redis_args());
		// println!("{:?}", redis::cmd("SET").arg(key.clone()).arg(&[5,6]));
		redis::Pipeline::new()
			.atomic()
			.del(key.clone()).ignore()
			.rpush(key.clone(), self.to_redis_args())
			.query(conn)
	}

	/// we need small transaction that figures out the vector length and gets range
	fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<Self>>
	{
		let llen = try!(conn.llen(key.clone()));
		conn.lrange(key.clone(), 0, llen)
	}
}

impl<K, X: RedisPersistency<K> + Ord + Default> RedisPersistency<K>
for BTreeSet<X>
where X: Default + protocol::ThriftTyped +
			redis::ToRedisArgs + redis::FromRedisValue + Sized + Hash + Clone,
	  K: redis::ToRedisArgs + Clone + Display {
	fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()>
	{
		// no implemenation for BTreeSet for `redis::ToRedisArgs`
		redis::Pipeline::new()
			.atomic()
			.del(key.clone()).ignore()
			.sadd(key.clone(), self.to_redis_args())
			.query(conn)
	}

	fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<Self>>
	{
		conn.smembers(key)
	}
}

impl<K, T: RedisPersistency<K> + Ord + Default,
	    V: RedisPersistency<K> > RedisPersistency<K>
for BTreeMap<T,V>
where T: Default + protocol::ThriftTyped +
			redis::ToRedisArgs + redis::FromRedisValue + Sized + Hash + Clone,
	  V: protocol::ThriftTyped +
	  		redis::ToRedisArgs + redis::FromRedisValue + Sized,
	  K: redis::ToRedisArgs + Clone + Display {
	fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()>
	{
		// no implemenation for BTreeSet for `redis::ToRedisArgs`
		redis::Pipeline::new()
			.atomic()
			.del(key.clone()).ignore()
			.cmd("HMSET").arg(key).arg(self.to_redis_args()).ignore()
			.query(conn)
	}

	fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<Self>>
	{
		conn.hgetall(key)
	}
}

impl<K, X: RedisPersistency<K> + Default> RedisPersistency<K> for Option<X>
where X: Default + protocol::ThriftTyped +
redis::ToRedisArgs + redis::FromRedisValue + Sized,
	  K: redis::ToRedisArgs + Clone + Display
{
	/// we delete key and write only if we have something to write, no key = None
	fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()>
	{
		try!(conn.del(key.clone()));
		self.as_ref().map(|this| this.redis_write(conn, key)).unwrap_or(Ok(()))
	}

	fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<Self>>
	{
		match conn.exists(key.clone()) {
			Err(e) => Err(e),
			Ok(false) => Ok(None),
			Ok(true) => {
				match X::redis_read(conn, key) {
					Err(e) => Err(e),
					Ok(v) => Ok(Some(v))
				}
			}
		}
	}
}

macro_rules! base_value_to_redis_impl {
    ($t:ty) => (

        impl<K> RedisPersistency<K> for $t
		where K: redis::ToRedisArgs + Clone + Display {
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()>
			{
				try!(conn.del(key.clone()));
				conn.set(key.clone(), self.to_redis_args())
			}

			/// we need small transaction that figures out the vector length and gets range
			fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<Self>>
			{
				conn.get(key)
			}
		}

    )
}

base_value_to_redis_impl!(String);
base_value_to_redis_impl!(i8);
base_value_to_redis_impl!(i16);
base_value_to_redis_impl!(i32);
base_value_to_redis_impl!(i64);
base_value_to_redis_impl!(f64);
base_value_to_redis_impl!(bool);
