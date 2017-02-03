use std::collections::{BTreeSet};
#[macro_use]
use customtraits;

strukt! {
    name = Simple,
    fields = {
        key: String => 16,
    }
}

strukt! {
    name = Empty,
    fields = {
    }
}

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
         six: Option<Simple> => 14,
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

