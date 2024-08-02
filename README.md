# Reflect
[![License](https://img.shields.io/crates/l/dimensionals)](https://choosealicense.com/licenses/mit/)

Reflect is a simple reflection library for Rust.   The idea is that:
- **types can be created** by name or from a ctor expression
- **methods can be invoked** given a: type instance, method name, and argument vector
- **static functions can be invoked** given a: type, static function name, and argument vector

The common use cases for this library are:

- **specification of type from configuration**
  * change behavior of application based on configuration, where configuration indicates a "ctor expression", for example `Sample(Momentum(SMA,[200,50,20], [0.20,0.30,0.50]), 900)`
  * such a type would implement a common Trait, so could be handled generically as `Box<dyn Trait>` and dispatched without knowledge of the specific type.
- **foreign interfaces**
  * creating rust types and calling methods from Python or other foreign interfaces

## Status
This is a work in progress, but should be feature complete (basic) within a week or so.  Core functionality is working.
 
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

// get ctor expression from some config source for example
let ctor_expr1 = config[&"signal/ctor"].to_string();
// inlined ctor here
let ctor_expr2 = "Momentum(SMA, [200,50,20], [0.20,0.30,0.50])";

// get the corresponding object
let obj = CTorParser::create::<BarFunction1D> (ctor_expr2);
```

For illustration purposes, we can also create with raw Rust.  This is tedious, but serves to illustrate aspects of what
the parser will do "under the covers":
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
In practice it would be pointless to evaluate in Rust as above.  The more typical use case is to:
- create a type dynamically via `CTorParser`
- map to a Trait
- call functions on the trait without specific knowledge of the underlying type or its parameterization
  

# Roadmap
The following is work in progress for a v0.1.0:

- [x] (impl) Type reflection
- [x] Enum reflection
- [x] Type conversion and equivalence
- [ ] Rework reflect_type to be reflect_impl and merge multiple impls into one type
- [ ] ctor parser
   
For future releases:
- [ ] handling of function signatures with Arc, Rc, and other type wrappers
- [ ] performance
