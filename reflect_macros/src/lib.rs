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
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemMethod, ReturnType, Type, TypePath, FnArg, Pat};
use proc_macro2::TokenStream as TokenStream2;


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
    let input = parse_macro_input!(item as ItemImpl);
    let type_name = &input.self_ty;

    // collect ctors and methods
    let mut constructor_registrations = vec![];
    let mut method_registrations = vec![];

    for item in &input.items {
        if let ImplItem::Method(method) = item {
            let registration = generate_registration(&method, &type_name);

            if is_constructor(&method) {
                constructor_registrations.push(registration);
            } else {
                method_registrations.push(registration);
            }
        }
    }

    eprintln!("ctor statement: {:?}", constructor_registrations.get(0).unwrap().to_string());

    let expanded = quote! {
        #input

        #(#constructor_registrations)*
        #(#method_registrations)*
    };

    expanded.into()
}


/// Get short-name for type
/// - we just want to get the human readable typename without the crate or module prefix
fn extract_short_type_name(type_name: &Type) -> proc_macro2::Ident {
    match type_name {
        Type::Path(TypePath { path, .. }) if !path.segments.is_empty() => {
            path.segments.last().unwrap().ident.clone()
        },
        _ => {
            panic!("Unsupported type in reflect_type");
        }
    }
}

/// Get full type string
fn get_type_path(ty: &Type) -> Option<proc_macro2::TokenStream> {
    if let Type::Path(type_path) = ty {
        Some(type_path.to_token_stream())
    } else {
        None
    }
}


/// Generate registration code
/// - for either a method or a ctor
fn generate_registration(method: &ImplItemMethod, type_name: &Box<Type>) -> TokenStream2 {
    let method_name = &method.sig.ident;
    let constructor_name = format_ident!("{}Constructor", ident_camel_case(method_name));
    let method_impl_name = format_ident!("{}Method", ident_camel_case(method_name));

    // get type name from token stream
    let short_type_name = extract_short_type_name(type_name);

    fn generate_arg_names(args: &[(proc_macro2::Ident, &Type)]) -> Vec<TokenStream2> {
        args.iter().map(|(name, _)| quote! { #name }).collect()
    }

    fn generate_arg_types(args: &[(proc_macro2::Ident, &Type)]) -> Vec<TokenStream2> {
        args.iter().map(|(_, ty)| quote! { std::any::TypeId::of::<#ty>() }).collect()
    }

    fn generate_arg_conversions(args: &[(proc_macro2::Ident, &Type)]) -> Vec<TokenStream2> {
        args.iter().enumerate().map(|(i, (name, ty))| {
            quote! {
                let #name = match args.get(#i).and_then(|arg| arg.downcast_ref::<#ty>()) {
                    Some(value) => *value,
                    None => return Err(format!("Invalid argument type for parameter {}", #i)),
                };
            }
        }).collect()
    }

    fn generate_return_info(output: &ReturnType) -> (TokenStream2, TokenStream2) {
        match output {
            ReturnType::Default => (quote! { () }, quote! { Ok(Box::new(())) }),
            ReturnType::Type(_, ty) => (quote! { #ty }, quote! { Ok(Box::new(result)) }),
        }
    }

    fn extract_args(method: &ImplItemMethod) -> Vec<(proc_macro2::Ident, &Type)> {
        method.sig.inputs.iter().filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some((pat_ident.ident.clone(), &*pat_type.ty))
                } else {
                    None
                }
            } else {
                None
            }
        }).collect()
    }


    let args = extract_args(method);
    let arg_conversions = generate_arg_conversions(&args);
    let arg_names = generate_arg_names(&args);
    let arg_types = generate_arg_types(&args);

    let (return_type, return_statement) = generate_return_info(&method.sig.output);

    if is_constructor(method) {
        generate_constructor_registration(
            &short_type_name, method_name, &constructor_name, &arg_conversions,
            &arg_names, &arg_types, &return_type, &return_statement)
    } else {
        generate_method_registration(
            type_name,
            &short_type_name, method_name, &method_impl_name, &arg_conversions,
            &arg_names, &arg_types, &return_type, &return_statement)
    }
}


/// Generate Constructor type
/// - this will be an instance of the `Constructor` trait and ultimately `Callable`
fn generate_constructor_registration(
    short_type_name: &proc_macro2::Ident,
    method_name: &proc_macro2::Ident,
    constructor_name: &proc_macro2::Ident,
    arg_conversions: &[TokenStream2],
    arg_names: &[TokenStream2],
    arg_types: &[TokenStream2],
    return_type: &TokenStream2,
    return_statement: &TokenStream2
) -> TokenStream2 {
    let register_ident = format_ident!("_REGISTER_{}", constructor_name);
    quote! {
        #[derive(Clone)]
        struct #constructor_name;

        impl ::reflect::Constructor for #constructor_name {
            fn create(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
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

            fn clone_boxed(&self) -> Box<dyn Constructor> {
                Box::new(self.clone())
            }
        }

        #[ctor::ctor]
        fn #register_ident () {
            ::reflect::register_constructor::<#short_type_name>(Box::new(#constructor_name));
        }
    }
}

/// Generate Method type
/// - this will be an instance of the `Method` trait and ultimately `Callable`
fn generate_method_registration(
    type_name: &Box<Type>,
    short_type_name: &proc_macro2::Ident,
    method_name: &proc_macro2::Ident,
    method_impl_name: &proc_macro2::Ident,
    arg_conversions: &[TokenStream2],
    arg_names: &[TokenStream2],
    arg_types: &[TokenStream2],
    return_type: &TokenStream2,
    return_statement: &TokenStream2
) -> TokenStream2 {
    let type_name = get_type_path(type_name).expect("expected a viable type");
    let register_ident = format_ident!("_REGISTER_{}", method_impl_name);

    quote! {
        #[derive(Clone)]
        struct #method_impl_name {
            name: String,
        }

        impl ::reflect::Method for #method_impl_name {
            fn call(&self, obj: &Box<dyn std::any::Any>, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                #(#arg_conversions)*

                let realobj = obj.downcast_ref::<#type_name>()
                    .expect("Failed to downcast to correct type");
                let result = realobj.#method_name(#(#arg_names),*);
                #return_statement
            }

            fn arg_types(&self) -> &[std::any::TypeId] {
                static ARG_TYPES: &[std::any::TypeId] = &[#(#arg_types),*];
                ARG_TYPES
            }

            fn return_type(&self) -> std::any::TypeId {
                std::any::TypeId::of::<#return_type>()
            }

            fn name(&self) -> &String {
                &self.name
            }

            fn clone_boxed(&self) -> Box<dyn Method> {
                Box::new(self.clone())
            }
        }

        #[ctor::ctor]
        fn #register_ident () {
            ::reflect::register_method::<#short_type_name>(Box::new(#method_impl_name {
                name: stringify!(#method_name).to_string(),
            }));
        }
    }
}


/// Determine if is ctor based on:
/// - return type is self
fn is_constructor(method: &ImplItemMethod) -> bool {
    fn is_self_type(rtype: &Type) -> bool {
        if let Type::Path(type_path) = rtype {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Self";
            }
        }
        false
    }

    // Check if the return type is Self
    matches!(method.sig.output, ReturnType::Type(_, ref ty) if is_self_type(ty))
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