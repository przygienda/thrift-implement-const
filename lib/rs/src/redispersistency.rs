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
pub trait RedisPersistency : Sized
{
	fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>;
	fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Self>>;
}

/// that's implementation if the type can be put into redis directly
impl<X> RedisPersistency for Vec<X>
where X: Default + protocol::ThriftTyped + redis::ToRedisArgs +
			redis::FromRedisValue + Sized,
{
	fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>
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
	fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Self>>
	{
		let llen = try!(conn.llen(key.clone()));
		conn.lrange(key.clone(), 0, llen)
	}
}

impl<X> RedisPersistency
for BTreeSet<X>
where X: RedisPersistency + Ord + Default + protocol::ThriftTyped +
			redis::ToRedisArgs + redis::FromRedisValue + Sized + Hash + Clone,
	   {
	fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>
	{
		// no implemenation for BTreeSet for `redis::ToRedisArgs`
		redis::Pipeline::new()
			.atomic()
			.del(key.clone()).ignore()
			.sadd(key.clone(), self.to_redis_args())
			.query(conn)
	}

	fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Self>>
	{
		conn.smembers(key)
	}
}

impl<T,V> RedisPersistency
for BTreeMap<T,V>
where T: RedisPersistency + Ord + Default + protocol::ThriftTyped +
			redis::ToRedisArgs + redis::FromRedisValue + Sized + Hash + Clone,
	  V: RedisPersistency + protocol::ThriftTyped +
	  		redis::ToRedisArgs + redis::FromRedisValue + Sized,
	   {
	fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>
	{
		// no implemenation for BTreeSet for `redis::ToRedisArgs`
		redis::Pipeline::new()
			.atomic()
			.del(key.clone()).ignore()
			.cmd("HMSET").arg(key).arg(self.to_redis_args()).ignore()
			.query(conn)
	}

	fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Self>>
	{
		conn.hgetall(key)
	}
}

impl<X> RedisPersistency for Option<X>
where X: RedisPersistency + Default + protocol::ThriftTyped +
		redis::ToRedisArgs + redis::FromRedisValue + Sized,

{
	/// we delete key and write only if we have something to write, no key = None
	fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>
	{
		try!(conn.del(key.clone()));
		self.as_ref().map(|this| this.redis_write(conn, key)).unwrap_or(Ok(()))
	}

	fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Self>>
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

        impl RedisPersistency for $t
		{
			fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>
			{
				try!(conn.del(key.clone()));
				conn.set(key.clone(), self.to_redis_args())
			}

			/// we need small transaction that figures out the vector length and gets range
			fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Self>>
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

// that makes the trait recursive, won't work since fields vary
//impl<T: RedisPersistency + RedisComposite> RedisPersistency for Vec<T>
//{
//	fn redis_write(&self, conn: &redis::Connection, key: &String) -> redis::RedisResult<()>
//	{
//		Ok(())
//	}
//	fn redis_read(conn: &redis::Connection, key: &String) -> redis::RedisResult<Option<Vec<Self>>>
//	{
//		Some(vec![])
//	}
//}
