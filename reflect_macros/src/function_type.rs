//! Determine type of function
//! - Constructor
//! - Method
//! - Static (static type-level function)
//!

use syn::{ImplItemMethod, Type, ReturnType};

/// Type of function
/// - our treatment of functions is somewhat different depending on whether is one of the three
///   categories
///
/// - for a `Constructor`, the function is like a static function except returns Self / Trait type
/// - for a `Method`, the function take a reference to &self, requiring an object reference
/// - for a `Static`, the function, like a ctor, does not take a reference to self and does not
///   need an object reference
#[derive(Clone, Copy)]
pub enum FunctionType {
    Constructor,
    Method,
    Static,
}

/// Determine the type of function given function AST
///
/// # Parameters
/// * `function`: the AST corresponding to the function
///
/// # Returns
/// * the type of function
pub fn determine_function_type(function: &ImplItemMethod) -> FunctionType {
    if function.sig.receiver().is_none() {
        match &function.sig.output {
            ReturnType::Type(_, ty) =>
                if is_self_or_impl_trait(ty) { FunctionType::Constructor } else { FunctionType::Static },
            ReturnType::Default =>
                FunctionType::Static,
        }
    } else {
        FunctionType::Method
    }
}

/// Determine whether the return type returns:
/// - Self OR
/// - Trait type
///
/// In the above two scenarios this function can be considered as a ctor
pub fn is_self_or_impl_trait(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) if type_path.path.is_ident("Self") => true,
        Type::ImplTrait(_) => true,
        _ => false,
    }
}