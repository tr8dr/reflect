use proc_macro2::Ident;
use quote::{quote, format_ident};
use syn::{Type, TypePath, TypeReference};
use crate::parser::{ParsedType, ParsedFunction};
use crate::function_type::FunctionType;
use crate::utilities::{ident_camel_case};


/// Generate code required for type reflection
/// - implementation of Function, Constructor, Method, or StaticFunction traits
/// - registration of type
/// - registration of functions
///
/// # Arguments
/// * `data`: the parsed information about the type
///
/// # Returns
///  * vector of token streams representing the generated code
pub fn generate_reflection_for_type(data: &ParsedType) -> Vec<proc_macro2::TokenStream> {
    data.functions.iter().map(|method| {
        match method.function_type {
            FunctionType::Constructor => generate_constructor(data, method),
            FunctionType::Method => generate_method(data, method),
            FunctionType::Static => generate_static(data, method),
        }
    }).collect()
}

/// Generates code for a constructor and registration
/// - implenentation of `Function` trait
/// - implenentation of `Constructor` trait
/// - registration
fn generate_constructor(data: &ParsedType, function: &ParsedFunction) -> proc_macro2::TokenStream {
    let short_type_name = &data.short_type_name;
    let method_name = &function.name;
    let ctor_name = format_ident!("{}Constructor", ident_camel_case(method_name));
    let register_ident = format_ident!("_REGISTER_{}", ctor_name);

    let (arg_conversions, arg_names, arg_types) = generate_arg_details(&function.args);
    let return_type = &function.return_type;

    quote! {
        #[derive(Clone)]
        struct #ctor_name {
            _arg_types: Vec<std::any::TypeId>
        }

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

        impl ::reflect::Constructor for #ctor_name {
            fn create(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                #(#arg_conversions)*
                let result = #short_type_name::#method_name(#(#arg_names),*);
                Ok(Box::new(result))
            }

            fn clone_boxed(&self) -> Box<dyn Constructor> {
                Box::new(self.clone())
            }
        }

        #[ctor::ctor]
        fn #register_ident() {
            ::reflect::register_constructor::<#short_type_name>(Box::new(#ctor_name {
                _arg_types: vec![#(#arg_types),*]
            }));
        }
    }
}

/// Generates code for a method and registration
/// - implenentation of `Function` trait
/// - implenentation of `Method` trait
/// - registration
fn generate_method(data: &ParsedType, function: &ParsedFunction) -> proc_macro2::TokenStream {
    let short_type_name = &data.short_type_name;
    let type_path = &data.type_path;
    let method_name = &function.name;
    let method_impl_name = format_ident!("{}Method", ident_camel_case(method_name));
    let register_ident = format_ident!("_REGISTER_{}", method_impl_name);

    let (arg_conversions, arg_names, arg_types) = generate_arg_details(&function.args);
    let return_type = &function.return_type;

    quote! {
        #[derive(Clone)]
        struct #method_impl_name {
            _name: String,
            _arg_types: Vec<std::any::TypeId>
        }

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

        impl ::reflect::Method for #method_impl_name {
            fn call(&self, obj: &Box<dyn std::any::Any>, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                #(#arg_conversions)*
                let realobj = obj.downcast_ref::<#type_path>().expect("Failed to downcast to correct type");
                let result = realobj.#method_name(#(#arg_names),*);
                Ok(Box::new(result))
            }

            fn clone_boxed(&self) -> Box<dyn Method> {
                Box::new(self.clone())
            }
        }

        #[ctor::ctor]
        fn #register_ident() {
            ::reflect::register_method::<#short_type_name>(Box::new(#method_impl_name {
                _name: stringify!(#method_name).to_string(),
                _arg_types: vec![#(#arg_types),*]
            }));
        }
    }
}

/// Generates code for a static function and registration
/// - implenentation of `Function` trait
/// - implenentation of `StaticFunction` trait
/// - registration
fn generate_static(data: &ParsedType, method: &ParsedFunction) -> proc_macro2::TokenStream {
    let short_type_name = &data.short_type_name;
    let method_name = &method.name;
    let fun_impl_name = format_ident!("{}Static", ident_camel_case(method_name));
    let register_ident = format_ident!("_REGISTER_{}", fun_impl_name);

    let (arg_conversions, arg_names, arg_types) = generate_arg_details(&method.args);
    let return_type = &method.return_type;

    quote! {
        #[derive(Clone)]
        struct #fun_impl_name {
            _name: String,
            _arg_types: Vec<std::any::TypeId>
        }

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

        impl ::reflect::StaticFunction for #fun_impl_name {
            fn call(&self, args: &[Box<dyn std::any::Any>]) -> Result<Box<dyn std::any::Any>, String> {
                #(#arg_conversions)*
                let result = #short_type_name::#method_name(#(#arg_names),*);
                Ok(Box::new(result))
            }

            fn clone_boxed(&self) -> Box<dyn StaticFunction> {
                Box::new(self.clone())
            }
        }

        #[ctor::ctor]
        fn #register_ident() {
            ::reflect::register_static::<#short_type_name>(Box::new(#fun_impl_name {
                _name: stringify!(#method_name).to_string(),
                _arg_types: vec![#(#arg_types),*]
            }));
        }
    }
}

/// Generate code for:
/// - argument conversions (from `Box<dyn Any>` to specific type for argument dispatch)
/// - argument namees
/// - argument type names
fn generate_arg_details(args: &[(syn::Ident, syn::Type)]) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let arg_conversions = args.iter().enumerate().map(|(i, (name, ty))| {
        generate_arg_conversion(i, name, ty)
    }).collect();

    let arg_names = args.iter().map(|(name, _)| quote! { #name }).collect();

    let arg_types = args.iter()
        .map(|(_, ty)| quote! { std::any::TypeId::of::<#ty>() })
        .collect();

    (arg_conversions, arg_names, arg_types)
}


/// Handle argument dereferencing dependent on type
///
/// # How this works
/// - creates a `let varname = deferenced value from args[i]`, for each argument
/// - this will later be placed in the body of the call function, so that the underlying
///   method can be dispatched
///
/// # Some ugliness
/// - some arguments will come in as `Vec<T>` whereas the receiving function will
///   usually take a slice: `&[T]`.  In this scenario, there is code to determine if the
///   target function parameter is a slice `&[T]` and if the incoming value is a `Vec[t]`
///   will get a slice on the `Vec[T]` argument
///
/// - aside from slices, there are references, primitive types, and struct based types.  There
///   may be some special handling for each in properly dereferencing
///
fn generate_arg_conversion(i: usize, name: &Ident, parameter_type: &Type) -> proc_macro2::TokenStream {
    match parameter_type {
        Type::Reference(TypeReference { elem, .. }) => {
            if let Type::Slice(_) = &**elem {
                // Handle &[T]
                quote! {
                    let #name = match args.get(#i) {
                        Some(arg) => {
                            if let Some(vec) = arg.downcast_ref::<Vec<_>>() {
                                vec.as_slice()
                            } else if let Some(slice) = arg.downcast_ref::<#parameter_type>() {
                                *slice
                            } else {
                                return Err(format!("Invalid argument type for parameter {}", #i));
                            }
                        },
                        None => return Err(format!("Missing argument for parameter {}", #i)),
                    };
                }
            } else {
                // Handle other reference types
                quote! {
                    let #name = match args.get(#i).and_then(|arg| arg.downcast_ref::<#parameter_type>()) {
                        Some(value) => *value,
                        None => return Err(format!("Invalid argument type for parameter {}", #i)),
                    };
                }
            }
        },
        Type::Path(TypePath { path, .. }) => {
            if path.segments.last().map_or(false, |seg| seg.ident == "Vec") {
                // Handle Vec<T>
                quote! {
                    let #name = match args.get(#i) {
                        Some(arg) => {
                            if let Some(vec) = arg.downcast_ref::<#parameter_type>() {
                                vec.clone()
                            } else {
                                return Err(format!("Invalid argument type for parameter {}", #i));
                            }
                        },
                        None => return Err(format!("Missing argument for parameter {}", #i)),
                    };
                }
            } else {
                // Handle primitive types
                quote! {
                    let #name = match args.get(#i).and_then(|arg| arg.downcast_ref::<#parameter_type>()) {
                        Some(value) => *value,
                        None => return Err(format!("Invalid argument type for parameter {}", #i)),
                    };
                }
            }
        },
        _ => {
            // Handle other types
            quote! {
                let #name = match args.get(#i).and_then(|arg| arg.downcast_ref::<#parameter_type>()) {
                    Some(value) => value.clone(),
                    None => return Err(format!("Invalid argument type for parameter {}", #i)),
                };
            }
        }
    }
}
