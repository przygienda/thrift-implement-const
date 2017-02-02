use std::collections::{BTreeSet};
use std::fmt::Display;
#[cfg(feature = "redis")]
use redis;

strukt! {
    name = Simple,
    fields = {
        key: String => 16,
    }
}

strukt! {
    name = Empty,
    fields = {}
}

// we don't implement arbitrary depth flatenning in Redis
#[cfg(not(feature = "redis"))]
strukt! {
    name = Nested,
    fields = {
        nested: Vec<Vec<Vec<Simple>>> => 32,
    }
}

strukt! {
    name = Recursive,
    fields = {
        recurse: Vec<Recursive> => 0,
    }
}

strukt! {
     name = Many,
     fields = {
         one: i32 => 3,
         two: String => 4,
         three: Vec<Simple> => 9,
         five: BTreeSet<Operation> => 11,
     }
}

strukt! {
    name = Optional,
    fields = {
        this: Option<i64> => 2,
    }
}

enom! {
    name = Operation,
    values = [
        Add = 1,
        Sub = 2,
        Clear = 3,
    ],
    default = Sub
}

