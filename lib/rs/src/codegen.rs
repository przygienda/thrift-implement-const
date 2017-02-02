#[macro_export]
macro_rules! service {
    (trait_name = $name:ident,
     processor_name = $processor_name:ident,
     client_name = $client_name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => $senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)*] ($srrty:ty),)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => $penname:ident = [$($pevname:ident($pename:ident: $pety:ty => $peid:expr),)*] ($prrty:ty),)*],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub trait $name {
            $(fn $smname(&self, $($saname: $saty),*) -> $srrty;)*
        }

        service_processor! {
            processor_name = $processor_name,
            service_methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*],
            parent_methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*],
            bounds = [$($boundty: $bound,)*],
            fields = [$($fname: $fty,)*]
        }

        service_client! {
            client_name = $client_name,
            service_methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*],
            parent_methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*]
        }
    }
}

#[macro_export]
macro_rules! service_processor {
    (processor_name = $name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => $senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)*] ($srrty:ty),)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => $penname:ident = [$($pevname:ident($pename:ident: $pety:ty => $peid:expr),)*] ($prrty:ty),)*],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub struct $name<$($boundty: $bound),*> {
            $($fname: $fty,)*
            proxies: $crate::proxy::Proxies
        }

        $(strukt! { name = $siname, fields = { $($saname: Option<$saty> => $said,)* } }
          strukt! { name = $soname, fields = { success: Option<$srty> => 0, $($sename: Option<$sety> => $seid,)* } }
          service_processor_error_enum! { $senname = [ $($sevname($sename: $sety => $seid),)*] })*

        impl<$($boundty: $bound),*> $name<$($boundty),*> {
            pub fn new($($fname: $fty),*) -> Self {
                $name { $($fname: $fname,)* proxies: Default::default() }
            }

            /// Add a `Proxy` to be used for all incoming messages.
            pub fn proxy<P>(&mut self, proxy: P)
            where P: 'static + Send + Sync + for<'e> $crate::proxy::Proxy<$crate::virt::VirtualEncodeObject<'e>> {
                self.proxies.proxy(proxy)
            }

            pub fn dispatch<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                                       name: &str, ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
                match name {
                    $(stringify!($smname) => self.$smname(prot, transport, ty, id),)*
                    $(stringify!($pmname) => self.$pmname(prot, transport, ty, id),)*
                    _ => Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            }

            service_processor_methods! { methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*] }
            service_processor_methods! { methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*] }
        }

        impl<P: $crate::Protocol, T: $crate::Transport, $($boundty: $bound),*> $crate::Processor<P, T> for $name<$($boundty),*> {
            fn process(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()> {
                #[allow(unused_imports)]
                use $crate::Protocol;

                let (name, ty, id) = try!(protocol.read_message_begin(transport));
                self.dispatch(protocol, transport, &name, ty, id)
            }
        }
    }
}

#[macro_export]
macro_rules! service_processor_error_enum {
    ($senname:ident = []) => {};
    ($senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)+]) => {
        #[derive(Debug, Clone)]
        pub enum $senname {
            $(
                $sevname($sety),
            )+
        }

        $(
            impl From<$sety> for $senname {
                fn from(v: $sety) -> $senname {
                    $senname::$sevname(v)
                }
            }
        )+
    }
}

#[macro_export]
macro_rules! service_processor_methods {
    (methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => $enname:ident = [$($evname:ident($ename:ident: $ety:ty => $eid:expr),)*] ($rrty:ty),)*]) => {
        $(fn $mname<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                               ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
            use $crate::proxy::Proxy;

            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            try!($crate::protocol::helpers::receive_body(prot, transport, MNAME,
                                                         &mut args, MNAME, ty, id));

            self.proxies.proxy(ty, MNAME, id, &args);

            // TODO: Further investigate this unwrap.
            let result = self.$fname.$mname($(args.$aname.unwrap()),*);
            let result = service_processor_methods_translate_return!(
                result, $oname, $enname = [$($evname($ename: $ety => $eid),)*]);
            try!($crate::protocol::helpers::send(prot, transport, MNAME,
                                                 $crate::protocol::MessageType::Reply, &result, id));

            Ok(())
        })*
    }
}

#[macro_export]
macro_rules! service_processor_methods_translate_return {
    ($result:expr, $oname:ident, $enname:ident = []) => {{
        let mut result = $oname::default();
        result.success = Some($result);
        result
    }};
    ($result:expr, $oname:ident, $enname:ident = [$($evname:ident($ename:ident: $ty:ty => $eid:expr),)+]) => {{
        let mut result = $oname::default();
        match $result {
            Ok(r) => result.success = Some(r),
            $(
                Err($enname::$evname(e)) => result.$ename = Some(e),
            )+
        }
        result
    }}
}

#[macro_export]
macro_rules! service_client {
    (client_name = $client_name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => $senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)*] ($srrty:ty),)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => $penname:ident = [$($pevname:ident($pename:ident: $pety:ty => $peid:expr),)*] ($prrty:ty),)*]) => {
        pub struct $client_name<P: $crate::Protocol, T: $crate::Transport> {
            pub protocol: P,
            pub transport: T
        }

        impl<P: $crate::Protocol, T: $crate::Transport> $client_name<P, T> {
            pub fn new(protocol: P, transport: T) -> Self {
                $client_name {
                    protocol: protocol,
                    transport: transport
                }
            }

            service_client_methods! { methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*] }
            service_client_methods! { methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*] }
        }
    }
}

#[macro_export]
macro_rules! service_client_methods {
    (methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => $enname:ident = [$($evname:ident($ename:ident: $ety:ty => $eid:expr),)*] ($rrty:ty),)*]) => {
        $(pub fn $mname(&mut self, $($aname: $aty,)*) -> $crate::Result<$rrty> {
            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            $(args.$aname = Some($aname);)*
            try!($crate::protocol::helpers::send(&mut self.protocol, &mut self.transport,
                                                 MNAME, $crate::protocol::MessageType::Call, &mut args, 0));

            let mut result = $oname::default();
            try!($crate::protocol::helpers::receive(&mut self.protocol, &mut self.transport,
                                                    MNAME, &mut result));

            let result = service_client_methods_translate_result!(
                result, $enname = [$($evname($ename: $ety => $eid),)*]);
            Ok(result)
        })*
    }
}

#[macro_export]
macro_rules! service_client_methods_translate_result {
    ($result:expr, $enname:ident = []) => {{
        use $crate::protocol::Encode;

        let result = $result;

        if result.success.should_encode() {
            result.success.unwrap()
        } else {
            result.success.unwrap_or_default()
        }
    }};
    ($result:expr, $enname:ident = [$($evname:ident($ename:ident: $ety:ty => $eid:expr),)*]) => {{
        let result = $result;
        if let Some(s) = result.success {
            Ok(s)
        }
        $(
            else if let Some(e) = result.$ename {
                Err($enname::$evname(e))
            }
        )*
        else {
            // TODO investigate this
            unreachable!()
        }
    }}
}

#[macro_export]
macro_rules! strukt {
    (name = $name:ident,
     fields = { $($fname:ident: $fty:ty => $id:expr,)+ }) => {
        #[derive(Debug, Clone, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
        pub struct $name {
            $(pub $fname: $fty,)+
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ(&self) -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Encode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::{Protocol};

                try!(protocol.write_struct_begin(transport, stringify!($name)));

                $(if $crate::protocol::Encode::should_encode(&self.$fname) {
                    try!(protocol.write_field_begin(transport, stringify!($fname),
                                                    $crate::protocol::helpers::typ::<$fty>(), $id));
                    try!($crate::protocol::Encode::encode(&self.$fname, protocol, transport));
                    try!(protocol.write_field_end(transport));
                })*

                try!(protocol.write_field_stop(transport));
                try!(protocol.write_struct_end(transport));

                Ok(())
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Decode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.read_struct_begin(transport));

                loop {
                    let (_, typ, id) = try!(protocol.read_field_begin(transport));

                    if typ == $crate::protocol::Type::Stop {
                        break;
                    } $(else if (typ, id) == ($crate::protocol::helpers::typ::<$fty>(), $id) {
                        try!($crate::protocol::Decode::decode(&mut self.$fname, protocol, transport));
                    })* else {
                        try!(protocol.skip(transport, typ));
                    }

                    try!(protocol.read_field_end(transport));
                }

                try!(protocol.read_struct_end(transport));

                Ok(())
            }
        }

        #[cfg(feature = "redis")]
		macro_rules! FIELDNDX { () => ( ":FIELD_{:04}")  }
		#[cfg(feature = "redis")]
		macro_rules! ARRAYSIZE { () => ( ":ARRAYSIZE")  }
		#[cfg(feature = "redis")]
		macro_rules! ARRAYNDX { () => ( ":INDEX_{:04}")  }

        // we normalize advanced sets over a vector of references for other collections
		#[cfg(feature = "redis")]
		impl $name {

		    fn redis_write_ref_vec<K>(vec: &Vec<&$name>, conn: &redis::Connection, key: K)
									-> redis::RedisResult<()>
			where K: redis::ToRedisArgs + Clone + Display {

				use redis::Commands;
				use $crate::redispersistency::RedisPersistency;

				let mut dk = format ! ("{}", key);
				let dklen = dk.len();

				{
					let mut az = dk.clone();
					az.push_str( & format ! (ARRAYSIZE ! ()));
					try ! (conn.set(az, vec.len()));
				}
				for e in vec.iter().enumerate() {
					dk.push_str( & format ! (ARRAYNDX ! (), e.0));
					try ! ((**e.1).redis_write(conn, dk.clone()));
					dk.truncate(dklen);
				}

				Ok(())
			}

			fn redis_read_vec<K>(conn: &redis::Connection, key: K)
				-> redis::RedisResult<Option<Vec<$name>>>
			where K: redis::ToRedisArgs + Clone + Display {
					use redis::Commands;

					let mut dk = format!("{}",key);
					let dklen = dk.len();

					// do NOT use scan or anything like this, Redis will just walk all keys
					// possibly
					dk.push_str(&format!(ARRAYSIZE!()));
					conn.get(dk.clone())
						.map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError,
															"cannot find array size")))
						.and_then(|alen| {
							if let Some(ialen) = alen {
								let mut rv = Vec::<$name>::with_capacity(ialen);

								dk.truncate(dklen);
								for _ in 0..ialen {
									let mut r: $ name = $ name::default();
									$ ({

										dk.push_str( & format ! (FIELDNDX ! (), $id));
										if let Some(v) = try ! (
											$crate::redispersistency::RedisPersistency::redis_read(conn,
												dk.clone())) {
											r.$fname = v;
										}
										dk.truncate(dklen);
									}) *
									rv.push(r);
								}
								Ok(Some(rv))
							} else {
								Err(redis::RedisError::from((redis::ErrorKind::TypeError,
															"cannot find array size")))
							}
						})
			}
		}

		#[cfg(feature = "redis")]
        impl<K> $crate::redispersistency::RedisPersistency<K> for $name
		where K: redis::ToRedisArgs + Clone + Display
		{
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()> {

				let mut dk = format!("{}",key);
				let dklen = dk.len();

				$({
					dk.push_str(&format!(FIELDNDX!(),$id));
					try!(self.$fname.redis_write(conn, dk.clone()));
					dk.truncate(dklen);
				})*
				Ok(())
			}

			fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<$name>> {
				let mut dk = format!("{}",key);
				let dklen = dk.len();

				let mut r : $name = $name::default();

				$({
					dk.push_str(&format!(FIELDNDX!(),$id));
					if let Some(v) = try!($crate::redispersistency::RedisPersistency::redis_read(conn,
															dk.clone())) {
						r.$fname = v;
					}
					dk.truncate(dklen);
				})*
				Ok(Some(r))
			}
		}

		#[cfg(feature = "redis")]
        impl<K> $crate::redispersistency::RedisPersistency<K> for Vec<$name>
		where K: redis::ToRedisArgs + Clone + Display
		{
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()> {
				$name::redis_write_ref_vec::<K>(&self.iter().collect::<Vec<_>>(), conn, key)
			}

			fn redis_read(conn: &redis::Connection, key: K)
				-> redis::RedisResult<Option<Vec<$name>>> {
				$name::redis_read_vec::<K>(conn, key)
			}
		}

		#[cfg(feature = "redis")]
        impl<K> $crate::redispersistency::RedisPersistency<K> for BTreeSet<$name>
		where K: redis::ToRedisArgs + Clone + Display
		{
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()> {
				$name::redis_write_ref_vec::<K>(&self.iter().collect::<Vec<_>>(), conn, key)
			}

			fn redis_read(conn: &redis::Connection, key: K)
				-> redis::RedisResult<Option<BTreeSet<$name>>> {
				if let Ok(rv) = $name::redis_read_vec::<K>(conn, key) {
					if let Some(irv) = rv {
						Ok(Some(irv.into_iter().collect::<BTreeSet<$name>>()))
					} else {
						Ok(None)
					}
				} else {
					Ok(None)
				}
			}
		}

		#[cfg(feature = "redis")]
		impl redis::FromRedisValue for $name {
			fn from_redis_value(_: &redis::Value) -> redis::RedisResult<$name> {
				unreachable!();
			}
		}
    };
    (name = $name:ident, fields = {}) => {
        #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name;

        impl $crate::protocol::ThriftTyped for $name {
            fn typ(&self) -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.write_struct_begin(transport, stringify!($name)));
                try!(protocol.write_field_stop(transport));
                try!(protocol.write_struct_end(transport));

                Ok(())
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.read_struct_begin(transport));

                let (_, ty, _) = try!(protocol.read_field_begin(transport));
                if ty != $crate::protocol::Type::Stop {
                     return Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }

                try!(protocol.read_struct_end(transport));

                Ok(())
            }
        }

    }
}

#[macro_export]
macro_rules! enom {
    (name = $name:ident,
     values = [$($vname:ident = $val:expr,)*],
     default = $dname:ident) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[repr(i32)]
        pub enum $name {
            $($vname = $val),*
        }

        impl Default for $name {
            fn default() -> Self { $name::$dname }
        }

        impl $crate::protocol::FromNum for $name {
            fn from_num(num: i32) -> Option<Self> {
                match num {
                    $($val => Some($name::$vname)),*,
                    _ => None
                }
            }
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ(&self) -> $crate::protocol::Type { $crate::protocol::Type::I32 }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                protocol.write_i32(transport, *self as i32)
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                *self = try!($crate::protocol::helpers::read_enum(protocol, transport));
                Ok(())
            }
        }

        #[cfg(feature = "redis")]
        impl<K> $crate::redispersistency::RedisPersistency<K> for $name
		where K: redis::ToRedisArgs + Clone + Display
		{
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()> {
				use redis::Commands;

				let dk = format!("{}",key);
				try!(conn.set(dk,*self as i32));
				Ok(())
			}

			fn redis_read(conn: &redis::Connection, key: K) -> redis::RedisResult<Option<$name>> {
				use redis::Commands;
				use $crate::protocol::FromNum;

				let dk = format!("{}",key);

				if let Some(v) = try!(conn.get(dk)) {
					Ok($name::from_num(v))
				} else {
					Ok(None)
				}

			}
		}

		#[cfg(feature = "redis")]
        impl<K> $crate::redispersistency::RedisPersistency<K> for Vec<$name>
		where K: redis::ToRedisArgs + Clone + Display
		{
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()> {
				self.iter().cloned().map(|e| e as i32).collect::<Vec<_>>().redis_write(conn,key)
			}

			fn redis_read(conn: &redis::Connection, key: K)
				-> redis::RedisResult<Option<Vec<$name>>> {
				use $crate::protocol::FromNum;

				if let Some(vec) = try!(Vec::<i32>::redis_read(conn,key)) {
					Ok(Some(vec
						.into_iter()
							.filter_map(|v| $name::from_num(v))
						.collect()))
				} else {
					Ok(None)
				}
			}
		}

		#[cfg(feature = "redis")]
        impl<K> $crate::redispersistency::RedisPersistency<K> for BTreeSet<$name>
		where K: redis::ToRedisArgs + Clone + Display
		{
			fn redis_write(&self, conn: &redis::Connection, key: K) -> redis::RedisResult<()> {
				self.iter().cloned().map(|e: $name| e as i32).collect::<Vec<_>>().redis_write(conn,key)
			}

			fn redis_read(conn: &redis::Connection, key: K)
				-> redis::RedisResult<Option<BTreeSet<$name>>> {
				use $crate::protocol::FromNum;

				if let Some(vec) = try!(Vec::<i32>::redis_read(conn,key)) {
					Ok(Some(vec
						.into_iter()
							.filter_map(|v| $name::from_num(v))
						.collect()))
				} else {
					Ok(None)
				}
			}
		}

		#[cfg(feature = "redis")]
		impl redis::FromRedisValue for $name {
			fn from_redis_value(_: &redis::Value) -> redis::RedisResult<$name> {
				unreachable!();
			}
		}
    }
}

