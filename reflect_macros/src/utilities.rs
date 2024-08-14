//! Miscellaneous utilities
//! - camel case (rust complains about types or traits not using camel case)
//! -
//!

use quote::{quote, format_ident};
use syn::Ident;


/// Convert to camel-case
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch.to_lowercase().next().unwrap());
        }
    }

    result
}

/// Convert identifier to camel-case
pub fn ident_camel_case(s: &proc_macro2::Ident) -> String {
    return to_camel_case(&s.to_string());
}