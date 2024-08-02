# Reflect
Reflect is a simple reflection library for Rust.   The idea is that:
- types can created by name or from a ctor expression
- methods invoked given type instance, method name, and argument vector
- static functions invoked given type, static function name, and argument vector

"Why?" you may ask.  The common use cases for this sort of facility are:

- specification of a type(args) in configuration, allowing for adjustment of behaviors at runtime
- creating rust types and calling methods from Python or other foreign interfaces

The library contains:
- a `#[reflect_type]` macro
  * used in decorating the `impl` of a type, allowing the type to be reflected
- a `#[reflect_enum]` macro
  * used in decorating an `enum` so that the enum implements `FromStr` and is known to reflection
- a reflection facility to:
  * create a named type given type name and arguments
  * call a method by name on a given type instance
  * call a static method by name on a given type
- a parser to parse "ctor expressions"
  * see below
