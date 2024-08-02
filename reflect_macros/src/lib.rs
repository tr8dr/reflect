//! The reflect library provides the following facilities:
//! - ability to register types for reflection
//! - fuzzy conversion functions, allowing for fuzzy matches in terms of argument matching
//!
//! The reflection macro finds ctors and method in an implementation of a type.  These
//! methods are then recorded and available to the reflection mechanism for type instance
//! creation and method invocation.
//!
//! Reflection allows for fuzzy matching for function arguments, where if, for example, a
//! vector of f64 is required and instead a vector of i64 is provided, if this is the best
//! fit method, before calling, the arguments will be transformed.
//!

mod generators;
mod utilities;
mod parser;
mod function_type;

use proc_macro::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::parse_macro_input;



/// Attribute to reflect ctors and methods in a type implementation
///
/// # Usage
/// ```
/// #[reflect_type]
/// impl MyType {
///     fn new (&self, a: f64, vec: &[i32]) -> &Self;
///
///     fn f (&self, x: f64) -> f64;
/// }
/// ```
///
/// This will generate:
/// - an implementation of Function as the base trait
/// - an implementation of Constructor for each ctor
/// - an implementation of Method for each method
/// - an implementation of StaticFunction for each type level function
/// - registration for each ctor, method, static function
/// - registration for the overall type
///
/// Given the above registration can then:
/// - create new `MyType` through reflection, yielding an object, say `obj`:
///   * `let obj = TypeInfo.create (args)`
/// - call methods by name
///   * `TypeInfo.call (obj, "f", arguments)`
///
/// The above is not terribly useful within Rust code, however when paired with a parser, from
/// configuration, python, etc. could have a constructed expression such as:
///
/// In json config
/// ```
/// {
///    "ctor": "MyType(3.149256, [200, 50, 20])"
/// }
/// ```
/// and use a parser (which we will provide) to construct types, nested types, etc. based on
/// expressions in configuration or from a scripting environment.
///
#[proc_macro_attribute]
pub fn reflect_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemImpl);
    let parsed_data = parser::parse_type_block (&input);
    let registrations = generators::generate_reflection_for_type (&parsed_data);

    quote! {
        #input
        #(#registrations)*
    }.into()
}
