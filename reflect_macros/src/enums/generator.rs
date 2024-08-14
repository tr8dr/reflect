//! Code generation for enum
//! - generation of FromStr trait
//! - generation of type conversion registration
//!

use proc_macro::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields};


/// Generate implementation of FromStr trait for enum
/// - generate `String` to `enum` field mappings
/// - implement `FromStr` on enum
///
/// We may want to check whether an implementation already exists OR allow user of macro to
/// provide a boolean in macro call
pub fn generate_enum_fromstr(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // get enum fields
    let fields = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("This macro can only be applied to enums"),
    };

    // conversion cases for match within from_str()
    let from_str_cases = fields.iter().map(|v| {
        let ident = &v.ident;
        let stringified = ident.to_string();
        match &v.fields {
            Fields::Unit => quote! { #stringified => Ok(Self::#ident) },
            _ => panic!("This macro only supports unit variants"),
        }
    });

    let expanded = quote! {
        impl #impl_generics std::str::FromStr for #name #ty_generics #where_clause {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_cases,)*
                    _ => Err(format!("Unknown variant: {}", s)),
                }
            }
        }
    };

    proc_macro2::TokenStream::from(expanded)
}


/// Generate enum type conversion registration
pub fn generate_enum_registration(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let register_ident = format_ident!("_REGISTER_{}", name);

    let expanded = quote! {
        #[ctor::ctor]
        fn #register_ident () {
            reflect::Conversions::add(
                std::any::TypeId::of::<String>(),
                std::any::TypeId::of::<#name>(),
                100,
                |v: &Box<dyn std::any::Any>| {
                    let s = v.downcast_ref::<String>().unwrap();
                    match #name::from_str(s) {
                        Ok(e) => Some(Box::new(e) as Box<dyn std::any::Any>),
                        Err(_) => None
                    }
                }
            );
        }
    };

    proc_macro2::TokenStream::from(expanded)
}