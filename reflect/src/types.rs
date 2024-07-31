
use std::any::{Any, TypeId};
use std::sync::Arc;


///
/// Constructor reflection information
///
pub trait Constructor: Send + Sync {
    /// call a ctor
    ///
    /// # Arguments
    /// * `args`: a list of arguments to the ctor
    ///
    /// # Returns
    /// * constructed instance
    fn create(&self, args: &[Box<dyn Any>]) -> Result<Box<dyn Any>, String>;

    /// Return the argument signature
    fn arg_types(&self) -> &[TypeId];

    /// The object type associated with this call
    fn return_type(&self) -> TypeId;

    /// create a boxed clone of this struct
    fn clone_boxed(&self) -> Box<dyn Constructor>;

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
pub trait Method: Send + Sync {
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

    /// Return the argument signature
    fn arg_types(&self) -> &[TypeId];

    /// The object type associated with this call
    fn return_type(&self) -> TypeId;

    /// create a boxed clone of this struct
    fn clone_boxed(&self) -> Box<dyn Method>;

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

}


/// TypeInfo requires clone in order to use Arc::make_mut
impl Clone for TypeInfo {
    fn clone(&self) -> Self {
        TypeInfo {
            name: self.name.clone(),
            objtype: self.objtype,
            constructors: self.constructors.iter().map(|c| c.clone_boxed()).collect(),
            methods: self.methods.iter().map(|m| (m.clone_boxed())).collect(),
        }
    }
}
