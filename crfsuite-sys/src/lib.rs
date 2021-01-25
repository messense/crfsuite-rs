#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

/// Force linking with libcqdb
#[doc(hidden)]
pub use libcqdb::*;

mod bindings;

pub use self::bindings::*;
