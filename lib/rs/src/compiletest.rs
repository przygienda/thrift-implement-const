#![allow(dead_code, non_camel_case_types)]
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

#[cfg(not(feature = "redis"))]
strukt! {
    name = DeeplyNested,
    fields = {
        nested: BTreeSet<Vec<Vec<Vec<Vec<i32>>>>> => 6,
    }
}

#[cfg(not(feature = "redis"))]
strukt! {
    name = ReferencesOther,
    fields = {
        other: DeeplyNested => 2,
        another: Simple => 3,
        map: BTreeMap<i32, Vec<String>> => 4,
    }
}

enom! {
    name = Operation,
    values = [Add = 1, Sub = 2, Mul = 3, Div = 4,],
    default = Add
}

#[cfg(not(feature = "redis"))]
service! {
    trait_name = SharedService,
    processor_name = SharedServiceProcessor,
    client_name = SharedServiceClient,
    service_methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult = shared.get_struct(key: i32 => 1,) -> DeeplyNested => SharedServiceGetStructError = [] (DeeplyNested),
    ],
    parent_methods = [],
    bounds = [S: SharedService,],
    fields = [shared: S,]
}

#[cfg(not(feature = "redis"))]
service! {
     trait_name = ChildService,
     processor_name = ChildServiceProcessor,
     client_name = ChildServiceClient,
     service_methods = [
         ChildServiceOperationArgs -> ChildServiceOperationResult = child.operation(
             one: String => 2,
             another: i32 => 3,
         ) -> Operation => ChildServiceOperationError = [] (Operation),
     ],
     parent_methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult = shared.get_struct(key: i32 => 1,) -> DeeplyNested => SharedServiceGetStructError = [] (DeeplyNested),
     ],
     bounds = [S: SharedService, C: ChildService,],
     fields = [shared: S, child: C,]
}

#[cfg(not(feature = "redis"))]
strukt! {
     name = Exception,
     fields = {
          name: String => 0,
          message: String => 1,
     }
}

#[cfg(not(feature = "redis"))]
service! {
    trait_name = ServiceWithException,
    processor_name = ServiceWithExceptionProcessor,
    client_name = ServiceWithExceptionClient,
    service_methods = [
        ServiceWithExceptionOperationArgs -> ServiceWithExceptionOperationResult = this.operation() -> i32 => ServiceWithExceptionOperationError = [Bad(bad: Exception => 1),] (Result<i32, ServiceWithExceptionOperationError>),
    ],
    parent_methods = [],
    bounds = [S: ServiceWithException,],
    fields = [this: S,]
}

