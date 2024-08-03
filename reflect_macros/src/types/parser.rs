//! AST Parsing functions
//! - AST-level representation of functions
//! - AST-level representation of type
//! - parsing of impl block -> abstract type representation
//!

use syn::{ItemImpl, ImplItem, Type, TypePath, Ident, ReturnType, FnArg, Pat, Path};
use quote::ToTokens;
use crate::types::function_type::{FunctionType, determine_function_type};


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
    pub trait_name: Option<Ident>,
    pub short_type_name: syn::Ident,
    pub type_path: proc_macro2::TokenStream,
    pub functions: Vec<ParsedFunction>,
}

/// Parse type (impl block)
/// - collect functions
/// - collect meta information about type
pub fn parse_type_block(input: &ItemImpl) -> ParsedType {
    let type_name = &input.self_ty;

    let (trait_id, type_id) = get_impl_info(&input);

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
        type_name: type_id,
        trait_name: trait_id,
        short_type_name,
        type_path,
        functions,
    }
}

/// Get type name and optional trait that is being implemented
/// - for a `impl Type` block the trait in (trait,type) will be None
/// - for a `impl Trait for Type` block the trait will have a value
fn get_impl_info(item: &ItemImpl) -> (Option<Ident>, Type) {
    // the rust AST interface is pretty nasty
    let trait_path = item.trait_.as_ref().and_then(|(_, path, _)| {
        path.segments.last().map(|seg| seg.ident.clone())
    });
    let type_path = (*item.self_ty).clone();

    (trait_path, type_path)
}