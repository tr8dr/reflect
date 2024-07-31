
use std::any::{Any, TypeId};
use std::sync::Arc;
use crate::{Constructor,Method,StaticFunction};


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
    pub methods: Vec<Box<dyn Method>>,
    pub functions: Vec<Box<dyn StaticFunction>>,
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
        let optctor = self.constructors.iter().find(|&ctor| {
           ctor.matching(args)
        });

        // check whether we found a ctor, then call
        match optctor {
            Some(&ref ctor) => {
                ctor.create(args)
            }
            None => {
                return Err(format!("could not find ctor for {} arguments", args.len()));
            }
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
        // find matching ctor (if any)
        let optmethod = self.methods.iter().find(|&method| {
           method.name() == name && method.matching(args)
        });

        // check whether we found a ctor, then call
        match optmethod {
            Some(&ref method) => {
                method.call(obj, args)
            }
            None => {
                return Err(format!("could not find method: '{}' for {} arguments", name, args.len()));
            }
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
        // find matching ctor (if any)
        let optfun = self.functions.iter().find(|&fun| {
           fun.name() == name && fun.matching(args)
        });

        // check whether we found a ctor, then call
        match optfun {
            Some(&ref fun) => {
                fun.call(name, args)
            }
            None => {
                return Err(format!("could not find function: '{}' for {} arguments", name, args.len()));
            }
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
            methods: self.methods.iter().map(|m| (m.clone_boxed())).collect(),
            functions: self.functions.iter().map(|m| (m.clone_boxed())).collect(),
        }
    }
}
