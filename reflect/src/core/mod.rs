//! This module contains low-level reflection machinery
//! - parts: `Constructor`, `Method`, `StaticFunction`
//! - representation of a type; `TypeInfo`
//! - registration
//!
//! See main library lib.rs for a more comprehensive description


mod types;
mod registration;
mod parts;
mod conversions;

pub use parts::{Constructor, Method, StaticFunction, Function};
pub use types::TypeInfo;
pub use conversions::Conversions;
pub use registration::{register_constructor, register_method, register_function, find_type};
