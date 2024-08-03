
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use crate::{Constructor, Conversions, Method, StaticFunction};


/// Information about a type
/// - name of type (short name as string)
/// - type id `TypeId`
/// - list of constructors
/// - list of methods
/// - list of functions
///
/// In addition, there are methods to:
/// - find type by name
/// - create type given ctor arguments
/// - call methods on type
/// - call static functions on type
///
pub struct TypeInfo {
    pub name: String,
    pub objtype: TypeId,
    pub constructors: Vec<Box<dyn Constructor>>,
    pub methods: HashMap<String,Box<dyn Method>>,
    pub functions: HashMap<String,Box<dyn StaticFunction>>,
}


/// Reflection interface to a type in the reflection system
/// - find type by name
/// - create type given ctor arguments
/// - call methods on type
/// - call static functions on type
impl TypeInfo {

    /// Find type associated with name
    ///
    /// # Arguments
    /// - `name`: type name (as string)
    ///
    /// # Returns
    /// - type info or `None`
    pub fn find_type(name: &str) -> Option<Arc<TypeInfo>> {
        crate::find_type (name)
    }

    /// Construct instance of this type given arguments
    ///
    /// # Arguments
    /// - `args`: arguments to ctor
    ///
    /// # Returns
    /// - new object instance (in the form of `Result<Box<dyn Any>, String>`)
    pub fn create (&self, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String> {
        // find matching ctor (if any)
        let ctor = match Conversions::find_best_match(&self.constructors, args) {
            Some(c) => c,
            None => return Err(format!("could not find ctor for {} arguments", args.len()))
        };
        let parameters = ctor.arg_types();

        // see if immediate match of arguments
        if ctor.matching(args) {
            ctor.create (args)
        }
        // otherwise need to convert arguments to be compatible
        else if Conversions::score (ctor.arg_types(), args) > 0 {
            match Conversions::convert_argv(parameters, args) {
                Some(newargs) => ctor.create (&newargs),
                None => Err(format!("incompatible arguments for ctor"))
            }

        } else {
            Err(format!("incompatible arguments for ctor"))
        }

    }

    /// Call method by name
    ///
    /// # Arguments
    /// - `name`: method name
    /// - `args`: arguments to ctor
    ///
    /// # Returns
    /// - method result `Result<Box<dyn Any>, String>`)
    pub fn call (&self, obj: &Box<dyn Any>, name: &str, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String> {
        // find matching method
        let method = match self.methods.get(name) {
            Some(m) => m,
            None => return Err(format!("could not find method: '{}'", name))
        };
        let parameters = method.arg_types();

        // see if immediate match of arguments
        if method.matching(args) {
            method.call(obj, args)
        }
        // otherwise need to convert arguments to be compatible
        else if Conversions::score (parameters, args) > 0 {
            match Conversions::convert_argv(parameters, args) {
                Some(newargs) => method.call (obj, &newargs),
                None => Err(format!("incompatible arguments for method: '{}'", name))
            }
        } else {
            Err(format!("incompatible arguments for method: '{}'", name))
        }
    }

    /// Call method by name
    ///
    /// # Arguments
    /// - `name`: method name
    /// - `args`: arguments to ctor
    ///
    /// # Returns
    /// - method result `Result<Box<dyn Any>, String>`)
    pub fn callstatic (&self, name: &str, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String> {
        // find matching static function
        let function = match self.functions.get(name) {
            Some(m) => m,
            None => return Err(format!("could not find function: '{}'", name))
        };
        let parameters = function.arg_types();

        // see if immediate match of arguments
        if function.matching(args) {
            function.call(args)
        }
        // otherwise need to convert arguments to be compatible
        else if Conversions::score (parameters, args) > 0 {
            match Conversions::convert_argv(parameters, args) {
                Some(newargs) => function.call (&newargs),
                None => Err(format!("incompatible arguments for function: '{}'", name))
            }
        } else {
            Err(format!("incompatible arguments for function: '{}'", name))
        }
    }

}


/// TypeInfo requires clone in order to use Arc::make_mut
impl Clone for TypeInfo {
    fn clone(&self) -> Self {
        TypeInfo {
            name: self.name.clone(),
            objtype: self.objtype,
            constructors: self.constructors.iter().map(|c| c.clone_boxed()).collect(),
            methods: self.methods.iter().map(|(k, v)| (k.clone(), v.clone_boxed())).collect(),
            functions: self.functions.iter().map(|(k, v)| (k.clone(), v.clone_boxed())).collect(),
        }
    }
}
