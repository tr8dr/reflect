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

mod types;
mod enums;
mod utilities;

use proc_macro::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields};


/// Attribute to reflect ctors and methods in a type implementation
///
/// # Usage
/// ```
/// #[reflect_impl]
/// impl Trait for MyType {
///     fn f (&self, x: f64) -> f64;
/// }
///
/// #[reflect_impl]
/// impl MyType {
///     fn new (&self, a: f64, vec: &[i32]) -> &Self;
///
///     fn g (&self, x: f64) -> f64;
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
///   * `TypeInfo.call (obj, "g", arguments)`
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
pub fn reflect_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemImpl);
    let parsed_data = types::parser::parse_type_block (&input);
    let registrations = types::generator::generate_reflection_for_type (&parsed_data);

    quote! {
        #input
        #(#registrations)*
    }.into()
}


/// Attribute to reflect enums
/// - allow enum creation from `String`
/// - registration of the `String` -> `enum` conversion
///
/// # Usage
/// Here is some example code:
/// ```
///   #[reflect_enum]
///   enum MAType {
///       SMA,
///       EMA,
///       KAMA
///   }
/// ```
///
/// The `reflect_enum` macro will generate an implementation of the `FromStr` trait
/// for the `MAType` enum and register it for conversion between `String` and `MAType`.
///
/// This comes in handy when instantiating a type from a ctor expression from config,
/// such as:  `"Momentum(SMA, [200, 50, 20], [0.20, 0.30, 0.50])"`.  In this expression
/// there would be a ctor for the `Momentum` type, expressed as:
///
/// ```
///    impl Momentum {
///        fn new (ma: MAType, windows: &[i32], weights: &[f64]) -> Self;
///    }
/// ```
///
/// The expression as parsed by the CTorParser will pass in the "SMA" parameter as a string
/// when it hands off for object creation.  Due to the conversion mapping between `String` and
/// `MAType`, the argyment will be converted to map to the appropriate enum.
///
/// Note that when trying to determine which ctor to call, the reflect library will score all
/// ctos relative to the arguments provided, and tries to find the best fit.   Conversions may
/// happen, as needed, if the match is not perfect.
///
#[proc_macro_attribute]
pub fn reflect_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;
    let fromstr = enums::generator::generate_enum_fromstr(&input);
    let register = enums::generator::generate_enum_registration(&input);

    let expanded = quote! {
        #input
        #(#fromstr)
        #(#register)
    };

    TokenStream::from(expanded)
}