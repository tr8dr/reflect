//! The reflect library provides the following facilities:
//! - ability to register types for reflection
//! - ability to find and create types from a string-based type id
//! - ability to call methods or static functions on a type via reflection
//!
//! In addition the crate also provides
//! - parsing for constructor expressions
//! - fuzzy type conversions in trying to match between an argument vector and a function
//!
//! # Registering a Type
//! Adding a type for reflection is accomplished as:
//! ```
//!    //#[reflect_type]
//!    impl Test1 {
//!        fn new (a: i32) -> Self {
//!            return Test1 { alpha: a };
//!        }
//!
//!        fn f(&self, x: i32) -> i32 {
//!            return x * self.alpha;
//!        }
//!    }
//! ```
//!
//! # Finding and Creating a Type
//! The `TypeInfo` struct has functions and method for reflecting a given type.  Finding
//! a type is accomplished as:
//! ```
//!    let itype = TypeInfo::find_type(&"Test1").expect("could not find type");
//! ```
//! One of the type's ctors can be invoked by matching an argument list with the signature
//! of one of the ctors.  The object instance is created as:
//! ```
//!    // create argv vector
//!    let args_ctor = vec![Box::new(42i32) as Box<dyn Any>];
//!    // find ctor and create obj
//!    let obj = itype.create(&args_ctor).expect("failed to call ctor");
//! ```
//! # Calling methods on an object
//! The `TypeInfo` struct has functions for calling methods and static functions.  A method is
//! called as:
//! ```
//!    // create argv vector
//!    let argv = vec![Box::new(3i32) as Box<dyn Any>];
//!    // call "f" method
//!    let result = TypeInfo::call (obj, "f", argv);
//! ```
//!


mod core;
mod parser;

pub use core::{Constructor, Method, StaticFunction, Function};
pub use core::TypeInfo;
pub use core::Conversions;
pub use core::{register_constructor, register_method, register_function, find_type};
pub use parser::CTorParser;


