# Reflect
Reflect is a simple reflection library for Rust.   The idea is that:
- types can created by name or from a ctor expression
- methods invoked given type instance, method name, and argument vector
- static functions invoked given type, static function name, and argument vector

The common use cases for this library are:

- **specification of type from configuration**
  * change behavior of application based on configuration, where configuration indicates a "ctor expression", for example `Sample(Momentum(SMA,[200,50,20], [0.20,0.30,0.50]), 900)`
- **foreign interfaces**
  * creating rust types and calling methods from Python or other foreign interfaces

## Functionality
The library provides:
- a `#[reflect_type]` macro
  * used in decorating the `impl` of a type, allowing the type to be reflected
- a `#[reflect_enum]` macro
  * used in decorating an `enum` so that the enum implements `FromStr` and is known to reflection
- a *reflection facility* to:
  * create a named type given type name and arguments
  * call a method by name on a given type instance
  * call a static method by name on a given type
- a parser to parse "ctor expressions"
  * able to parse and create type instances associated with expressions such as `Sample(Momentum(SMA,[200,50,20], [0.20,0.30,0.50]), 900)`
 
## Example
Here is an example type `Momentum` and an enum `MAType` that are to be reflected:
```rust
use reflect::{reflect_enum, reflect_type};

#[reflect_enum]
enum MAType {
    SMA,
    EMA,
    KAMA
}

type Momentum: BarFunction1D { ... }

#[reflect_type]
impl Momentum {
    // ctor
    fn new (ma: MAType, windows: &[i32], weights: &[f64]) -> Self;

    // a method
    fn tick (bar: &Bar) -> f32;

    // a static function
    fn evaluate (df: &DataFrame) -> Vec[f64];
}
```

Given a "ctor expression" from configuration, python, or another source, can create an instance of the type as:
```rust
use reflect::CTorParser;
...

// get ctor expression from some source (returning: "Momentum(SMA, [200,50,20], [0.20,0.30,0.50])")
let ctor_expr = config[&"ctor"];
// get the corresponding object
let obj = CTorParser::create::<BarFunction1D> (ctor_expr);
```

For illustration purposes, we can also create without parsing in Rust as:
```rust
use reflect::TypeInfo;
...
let windows = vec![200,50,20];
let weights = vec![0.20,0.30,0.50];
let ctor_args = vec![
    Box::new(Momentum::SMA) as Box<dyn Any>,
    Box::new(&windows) as Box<dyn Any>,
    Box::new(&weights) as Box<dyn Any>,
];

// find type by name
let type = TypeInfo::find("Momentum").expect("could not find type");
// evaluate ctor matching argument set
let obj = type.create (&ctor_args).expect("failed to find ctor");

// call method
let bar = Bar { ... };
let method_args = vec![ &bar ];
let result = type.call (obj, "tick", method_args);
```
In practice it would be pointless to evaluate as above.  More often the use case is the creation of a type and then direct
evaluation of a `Box<dyn Trait>` where the underlying specific type and instance was created with a ctor expression.

# Roadmap
The following planned for future releases:

- [*] Type reflection
- [*] Enum reflection
- [*] Type conversion and equivalence
- [ ] ctor parser (*work in progress*)
- [ ] navigating trait extensions (if possible)
- [ ] handling of function signatures with Arc, Rc, and other type wrappers
- [ ] performance
