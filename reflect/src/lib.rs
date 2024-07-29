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
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemMethod, ReturnType, Type, TypePath, FnArg, Pat};

mod types;
mod registration;

pub use types::{Constructor, Method, TypeInfo};
pub(crate) use registration::{register_constructor, register_method, find_type};


//
// Root level functions (unfortunately macros must be defined here)
//

/// Attribute to reflect ctors and methods in a type implementation
///
/// # Usage
/// ```
/// #[reflect_type]
/// impl MyType {
///     fn new (&self, a: f64) -> &Self;
///
///     fn f (&self, x: f64) -> f64;
/// }
/// ```
/// In the above example the constructor and the method `f` would be registered and made available
/// for reflection.
///
#[proc_macro_attribute]
pub fn reflect_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemImpl);

    // Extract the type name
    let type_name = &input.self_ty;
    let short_type_name = match &**type_name {
        Type::Path(TypePath { path, .. }) if !path.segments.is_empty() => {
            path.segments.last().unwrap().ident.clone()
        },
        _ => {
            return syn::Error::new_spanned(type_name, "Unsupported type in reflect_type")
                .to_compile_error()
                .into();
        }
    };

    let mut constructor_registrations = vec![];
    let mut method_registrations = vec![];

    // Iterate over each function in the type implementation
    for item in &input.items {
        if let ImplItem::Method(method) = item {
            let method_name = &method.sig.ident;
            let constructor_name = format_ident!("{}Constructor", method_name);
            let method_impl_name = format_ident!("{}Method", method_name);

            // Extract method arguments
            let args = method.sig.inputs.iter().filter_map(|arg| {
                match arg {
                    FnArg::Typed(pat_type) => {
                        let name = match &*pat_type.pat {
                            Pat::Ident(pat_ident) => &pat_ident.ident,
                            _ => return None,
                        };
                        Some((name, &pat_type.ty))
                    },
                    FnArg::Receiver(_) => None,
                }
            }).collect::<Vec<_>>();

            // Generate argument conversions
            let arg_conversions = args.iter().enumerate().map(|(i, (name, ty))| {
                quote! {
                    let #name = match args.get(#i).and_then(|arg| arg.downcast_ref::<#ty>()) {
                        Some(value) => value,
                        None => return Err(format!("Invalid argument type for parameter {}", #i)),
                    };
                }
            });

            // Generate argument names for method call
            let arg_names = args.iter().map(|(name, _)| quote! { #name });

            // Generate argument types for reflection
            let arg_types = args.iter().map(|(_, ty)| quote! { std::any::TypeId::of::<#ty>() });

            // Determine if the method is a constructor
            let is_ctor = is_constructor(method);

            // Generate the return type handling
            let (return_type, return_statement) = match &method.sig.output {
                ReturnType::Default => (quote! { () }, quote! { Ok(Box::new(())) }),
                ReturnType::Type(_, ty) => (quote! { #ty }, quote! { Ok(Box::new(result)) }),
            };

            let registration = if is_ctor {
                quote! {
                    #[derive(Clone)]
                    struct #constructor_name;

                    impl Callable for #constructor_name {
                        fn call(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                            #(#arg_conversions)*
                            let result = #short_type_name::#method_name(#(#arg_names),*);
                            #return_statement
                        }

                        fn arg_types(&self) -> &[std::any::TypeId] {
                            static ARG_TYPES: &[std::any::TypeId] = &[#(#arg_types),*];
                            ARG_TYPES
                        }

                        fn return_type(&self) -> std::any::TypeId {
                            std::any::TypeId::of::<#return_type>()
                        }
                    }

                    impl Constructor for #constructor_name {
                        fn clone_boxed(&self) -> Box<dyn Constructor> {
                            Box::new(self.clone())
                        }
                    }

                    register_constructor::<#type_name>(Box::new(#constructor_name));
                }
            } else {
                quote! {
                    #[derive(Clone)]
                    struct #method_impl_name {
                        name: String,
                    }

                    impl Callable for #method_impl_name {
                        fn call(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                            #(#arg_conversions)*
                            let result = #short_type_name::#method_name(#(#arg_names),*);
                            #return_statement
                        }

                        fn arg_types(&self) -> &[std::any::TypeId] {
                            static ARG_TYPES: &[std::any::TypeId] = &[#(#arg_types),*];
                            ARG_TYPES
                        }

                        fn return_type(&self) -> std::any::TypeId {
                            std::any::TypeId::of::<#return_type>()
                        }
                    }

                    impl Method for #method_impl_name {
                        fn name(&self) -> &String {
                            &self.name
                        }

                        fn clone_boxed(&self) -> Box<dyn Method> {
                            Box::new(self.clone())
                        }
                    }


                    register_method::<#type_name>(Box::new(#method_impl_name {
                        name: stringify!(#method_name).to_string(),
                    }));
                }
            };

            if is_ctor {
                constructor_registrations.push(registration);
            } else {
                method_registrations.push(registration);
            }
        }
    }

    let expanded = quote! {
        #input

        #(#constructor_registrations)*
        #(#method_registrations)*
    };

    TokenStream::from(expanded)
}


fn is_constructor(method: &ImplItemMethod) -> bool {
    // Check if the method name starts with "new"
    method.sig.ident.to_string().starts_with("new") &&
    // Check if the return type is Self
    matches!(method.sig.output, ReturnType::Type(_, ref ty) if is_self_type(ty))
}

fn is_self_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Self";
        }
    }
    false
}
