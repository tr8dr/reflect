
use std::any::{Any, TypeId};
use std::sync::Arc;

///
/// Callable Function
///
pub trait Function: Send + Sync {
    /// Return the argument signature
    fn arg_types(&self) -> &[TypeId];

    /// The object type associated with this call
    fn return_type(&self) -> TypeId;

    /// Determine if arguments match this callable
    ///
    /// # Arguments
    /// - `args`: array of arguments
    fn matching(&self, args: &[Box<dyn Any>]) -> bool {
        let arg_types = self.arg_types();

        // Check arity (does the number of arguments match?)
        if arg_types.len() != args.len() {
            return false;
        }

        // Check if each argument type matches
        arg_types.iter().zip(args.iter()).all(|(expected_type, arg)| {
            let actual_type = (**arg).type_id();
            actual_type == *expected_type
        })
    }

}


///
/// Constructor reflection information
///
pub trait Constructor: Function {
    /// call a ctor
    ///
    /// # Arguments
    /// * `args`: a list of arguments to the ctor
    ///
    /// # Returns
    /// * constructed instance
    fn create(&self, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String>;

    /// create a boxed clone of this struct
    fn clone_boxed(&self) -> Box<dyn Constructor>;
}


///
/// Constructor reflection information
///
pub trait Method: Function {
    /// method name
    fn name(&self) -> &String;

    /// call a method on object
    ///
    /// # Arguments
    /// * `obj`: object on which the method should be called
    /// * `args`: a list of arguments to the ctor
    ///
    /// # Returns
    /// * function value
    fn call(&self, obj: &Box<dyn Any>, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String>;

    /// create a boxed clone of this struct
    fn clone_boxed(&self) -> Box<dyn Method>;
}


///
/// static function reflection information
///
pub trait Static: Function {
    /// function name
    fn name(&self) -> &String;

    /// call a ctor
    ///
    /// # Arguments
    /// * `name`: name of function
    /// * `args`: a list of arguments to the ctor
    ///
    /// # Returns
    /// * constructed instance
    fn call(&self, name: &str, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String>;

    /// create a boxed clone of this struct
    fn clone_boxed(&self) -> Box<dyn Static>;
}


//
// Concrete types
//

///
/// Information about a type
///
pub struct TypeInfo {
    pub name: String,
    pub objtype: TypeId,
    pub constructors: Vec<Box<dyn Constructor>>,
    pub methods: Vec<Box<dyn Method>>,
    pub functions: Vec<Box<dyn Static>>,
}

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
