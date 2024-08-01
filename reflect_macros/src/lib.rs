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

use proc_macro::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{parse_macro_input, ItemImpl, ImplItem, ReturnType, Type, TypePath, FnArg, Pat};


/// Types of functions we may encounter in a type
enum FunctionType {
    Constructor,
    Method,
    Static,
}


/// Attribute to reflect ctors and methods in a type implementation
///
/// # Usage
/// ```
/// #[reflect_type]
/// impl MyType {
///     fn new (&self, a: f64, b: f64) -> &Self;
///
///     fn f (&self, x: f64) -> f64;
/// }
/// ```
///
/// This will generate:
/// - an implementation of Constructor for each ctor
/// - an implementation of Method for each method
/// - registration for each ctor and method
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
///    "ctor": "MyType(42, 3.1415926)"
/// }
/// ```
/// and use a parser (which we will provide) to construct types, nested types, etc. based on
/// expressions in configuration or from a scripting environment.
///
#[proc_macro_attribute]
pub fn reflect_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
let input = parse_macro_input!(item as ItemImpl);
    let type_name = &input.self_ty;

    // get type short name
    let short_type_name = match type_name.as_ref() {
        Type::Path(TypePath { path, .. }) if !path.segments.is_empty() => path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type in reflect_type"),
    };

    // get type name
    let type_path = if let Type::Path(type_path) = type_name.as_ref() {
        type_path.to_token_stream()
    } else {
        panic!("Expected a viable type")
    };

    // Helper function to check if the return type is Self or impl Trait
    fn is_self_or_impl_trait(ty: &Type) -> bool {
        match ty {
            Type::Path(type_path) if type_path.path.is_ident("Self") => true,
            Type::ImplTrait(_) => true,
            _ => false,
        }
    }

    // create code for each ctor or method
    let registrations = input.items.iter().filter_map(|item| {
        if let ImplItem::Method(method) = item {
            let method_name = &method.sig.ident;

            // Determine if this is a constructor, instance method, or static method
            let function_type = if method.sig.receiver().is_none() {
                match &method.sig.output {
                    ReturnType::Type(_, ty) =>
                        if is_self_or_impl_trait(ty) { FunctionType::Constructor } else { FunctionType::Static },
                    ReturnType::Default =>
                        FunctionType::Static,
                }
            } else {
                FunctionType::Method
            };

            // get arguments for function
            let args = method.sig.inputs.iter()
                .filter_map(|arg| if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        Some((pat_ident.ident.clone(), &*pat_type.ty))
                    } else { None }
                } else { None })
                .collect::<Vec<_>>();

            let arg_conversions = args.iter().enumerate().map(|(i, (name, ty))| quote! {
                let #name = match args.get(#i).and_then(|arg| arg.downcast_ref::<#ty>()) {
                    Some(value) => *value,
                    None => return Err(format!("Invalid argument type for parameter {}", #i)),
                };
            }).collect::<Vec<_>>();

            // names of arguments
            let arg_names = args.iter().map(|(name, _)| quote! { #name }).collect::<Vec<_>>();
            // type ids of arguments
            let (return_type, return_statement) = match &method.sig.output {
                ReturnType::Default => (quote! { () }, quote! { Ok(Box::new(())) }),
                ReturnType::Type(_, ty) => (quote! { #ty }, quote! { Ok(Box::new(result)) }),
            };

            let arg_types = args.iter()
                .map(|(_, ty)| quote! { std::any::TypeId::of::<#ty>() })
                .collect::<Vec<_>>();

            match function_type {
                // constructor functions
                FunctionType::Constructor => {
                    let ctor_name = format_ident!("{}Constructor", ident_camel_case(method_name));
                    let register_ident = format_ident!("_REGISTER_{}", ctor_name);

                    Some(quote! {
                        /// specific Constructor type
                        #[derive(Clone)]
                        struct #ctor_name {
                            _arg_types: Vec<std::any::TypeId>
                        }

                        /// implementation of Function trait for the given ctor
                        impl ::reflect::Function for #ctor_name {
                            fn name(&self) -> &str {
                                &"*"
                            }

                            fn arg_types(&self) -> &[std::any::TypeId] {
                                &self._arg_types
                            }

                            fn return_type(&self) -> std::any::TypeId {
                                std::any::TypeId::of::<#return_type>()
                            }
                        }

                        /// implementation of Constructor trait for the given ctor
                        impl ::reflect::Constructor for #ctor_name {
                            fn create(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                                #(#arg_conversions)*
                                let result = #short_type_name::#method_name(#(#arg_names),*);
                                #return_statement
                            }

                            fn clone_boxed(&self) -> Box<dyn Constructor> {
                                Box::new(self.clone())
                            }
                        }

                        /// auto-registration function
                        #[ctor::ctor]
                        fn #register_ident() {
                            ::reflect::register_constructor::<#short_type_name>(Box::new(#ctor_name {
                                _arg_types: vec![#(#arg_types),*]
                            }));
                        }
                    })
                }

                // normal method functions
                FunctionType::Method => {
                    let method_impl_name = format_ident!("{}Method", ident_camel_case(method_name));
                    let register_ident = format_ident!("_REGISTER_{}", method_impl_name);

                    Some(quote! {
                        /// specific Method type
                        #[derive(Clone)]
                        struct #method_impl_name {
                            _name: String,
                            _arg_types: Vec<std::any::TypeId>
                        }

                        /// implementation of Function trait for the given method
                        impl ::reflect::Function for #method_impl_name {
                            fn name(&self) -> &str {
                                &self._name
                            }

                            fn arg_types(&self) -> &[std::any::TypeId] {
                                &self._arg_types
                            }

                            fn return_type(&self) -> std::any::TypeId {
                                std::any::TypeId::of::<#return_type>()
                            }
                        }

                        /// implementation of Method trait for the given method
                        impl ::reflect::Method for #method_impl_name {
                            fn call(&self, obj: &Box<dyn std::any::Any>, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                                #(#arg_conversions)*
                                let realobj = obj.downcast_ref::<#type_path>().expect("Failed to downcast to correct type");
                                let result = realobj.#method_name(#(#arg_names),*);
                                #return_statement
                            }

                            fn clone_boxed(&self) -> Box<dyn Method> {
                                Box::new(self.clone())
                            }
                        }

                        /// auto-registration function
                        #[ctor::ctor]
                        fn #register_ident() {
                            ::reflect::register_method::<#short_type_name>(Box::new(#method_impl_name {
                                _name: stringify!(#method_name).to_string(),
                                _arg_types: vec![#(#arg_types),*]
                            }));
                        }
                    })
                }

                // temporarily do not implement static functions
                FunctionType::Static => {
                    let fun_impl_name = format_ident!("{}Static", ident_camel_case(method_name));
                    let register_ident = format_ident!("_REGISTER_{}", fun_impl_name);

                    Some(quote! {
                        /// specific Method type
                        #[derive(Clone)]
                        struct #fun_impl_name {
                            _name: String,
                            _arg_types: Vec<std::any::TypeId>
                        }

                        /// implementation of Function trait for the given method
                        impl ::reflect::Function for #fun_impl_name {
                            fn name(&self) -> &str {
                                &self._name
                            }

                            fn arg_types(&self) -> &[std::any::TypeId] {
                                &self._arg_types
                            }

                            fn return_type(&self) -> std::any::TypeId {
                                std::any::TypeId::of::<#return_type>()
                            }
                        }

                        /// implementation of Method trait for the given method
                        impl ::reflect::StaticFunction for #fun_impl_name {
                            fn call(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                                #(#arg_conversions)*
                                let result = #short_type_name::#method_name(#(#arg_names),*);
                                #return_statement
                            }

                            fn clone_boxed(&self) -> Box<dyn Method> {
                                Box::new(self.clone())
                            }
                        }

                        /// auto-registration function
                        #[ctor::ctor]
                        fn #register_ident() {
                            ::reflect::register_static::<#short_type_name>(Box::new(#fun_impl_name {
                                _name: stringify!(#method_name).to_string(),
                                _arg_types: vec![#(#arg_types),*]
                            }));
                        }
                    })
                }
            }
        } else {
            None
        }
    }).collect::<Vec<_>>();

    //eprintln!("first: {:?}", registrations.last().unwrap().to_string());

    quote! {
        #input
        #(#registrations)*
    }.into()
}



/// Convert to camel-case
fn to_camel_case(s: &str) -> String {
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
fn ident_camel_case(s: &proc_macro2::Ident) -> String {
    return to_camel_case(&s.to_string());
}