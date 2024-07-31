
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
pub trait StaticFunction: Function {
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
    fn clone_boxed(&self) -> Box<dyn StaticFunction>;
}

