//! AST Parsing functions
//! - AST-level representation of functions
//! - AST-level representation of type
//! - parsing of impl block -> abstract type representation
//!

use syn::{ItemImpl, ImplItem, Type, TypePath, ReturnType, FnArg, Pat};
use quote::ToTokens;
use crate::function_type::{FunctionType, determine_function_type};


/// Representation of a function
/// - name of function (important for methods and static functions)
/// - type of function (Constructor, Method, Static)
/// - argument vector of (name, type)
/// - function return type
pub struct ParsedFunction {
    pub name: syn::Ident,
    pub function_type: FunctionType,
    pub args: Vec<(syn::Ident, syn::Type)>,
    pub return_type: syn::Type,
}

/// AST-level representation of a type
pub struct ParsedType {
    pub type_name: syn::Type,
    pub short_type_name: syn::Ident,
    pub type_path: proc_macro2::TokenStream,
    pub functions: Vec<ParsedFunction>,
}

/// Parse type (impl block)
/// - collect functions
/// - collect meta information about type
pub fn parse_type_block(input: &ItemImpl) -> ParsedType {
    let type_name = &input.self_ty;

    let short_type_name = match type_name.as_ref() {
        Type::Path(TypePath { path, .. }) if !path.segments.is_empty() => path.segments.last().unwrap().ident.clone(),
        _ => panic!("Unsupported type in reflect_type"),
    };

    let type_path = if let Type::Path(type_path) = type_name.as_ref() {
        type_path.to_token_stream()
    } else {
        panic!("Expected a viable type")
    };

    let functions = input.items.iter().filter_map(|item| {
        if let ImplItem::Method(method) = item {
            let function_type = determine_function_type(method);

            let args = method.sig.inputs.iter()
                .filter_map(|arg| if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        Some((pat_ident.ident.clone(), (*pat_type.ty).clone()))
                    } else { None }
                } else { None })
                .collect();

            let return_type = match &method.sig.output {
                ReturnType::Default => syn::parse_quote!(()),
                ReturnType::Type(_, ty) => (**ty).clone(),
            };

            Some(ParsedFunction {
                name: method.sig.ident.clone(),
                function_type,
                args,
                return_type,
            })
        } else {
            None
        }
    }).collect();

    ParsedType {
        type_name: (*input.self_ty).clone(),
        short_type_name,
        type_path,
        functions,
    }
}