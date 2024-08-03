
use std::any::{TypeId};
use std::any::type_name;

use crate::core::{Constructor, Method, StaticFunction};
use crate::core::TypeInfo;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;


//
// Repository of reflected types
//
lazy_static! {
    static ref TYPE_REGISTRY: Mutex<HashMap<String, Arc<TypeInfo>>> = Mutex::new(HashMap::new());
}


/// Get shortened type name for a given type
/// - avoids crate and module in the type so can use a more human naming
pub fn type_shortname<T: 'static>() -> String {
    // get short name for type (trimming off the crate and module)
    let type_name = type_name::<T>();
    type_name.split("::").last().unwrap_or(type_name).to_string()
}

/// Get type information for given named type
///
/// # Arguments
/// - `name`: name of type (as string)
///
/// # Returns
/// - `Some(typeinfo)` OR
/// - `None`
pub fn find_type(name: &str) -> Option<Arc<TypeInfo>> {
    let registry = TYPE_REGISTRY.lock().unwrap();
    match registry.get(name) {
        Some(info) => Some(info.clone()),
        None => None
    }
}


/// Register a constructor for a given type
///
/// # Arguments
/// - `constructor`: constructor to be added
pub fn register_constructor<T: 'static>(constructor: Box<dyn Constructor>) {
    let mut registry = TYPE_REGISTRY.lock().unwrap();
    let short_name = type_shortname::<T>();

    // get type associated with this ctor (or create type entry)
    let type_info = registry.entry(short_name.clone()).or_insert_with(|| {
        Arc::new(TypeInfo {
            name: short_name,
            objtype: TypeId::of::<T>(),
            constructors: Vec::new(),
            methods: HashMap::new(),
            functions: HashMap::new()
        })
    });

    Arc::make_mut(type_info).constructors.push(constructor);
}

/// Register a method for a given type
///
/// # Arguments
/// - `method`: method to be added
pub fn register_method<T: 'static>(method: Box<dyn Method>) {
    let mut registry = TYPE_REGISTRY.lock().unwrap();
    let short_name = type_shortname::<T>();

    // get type associated with this method (or create type entry)
    let type_info = registry.entry(short_name.clone()).or_insert_with(|| {
        Arc::new(TypeInfo {
            name: short_name,
            objtype: TypeId::of::<T>(),
            constructors: Vec::new(),
            methods: HashMap::new(),
            functions: HashMap::new()
        })
    });

    let key = method.name().to_string();
    Arc::make_mut(type_info).methods.insert(key, method);
}


/// Register a static function for a given type
///
/// # Arguments
/// - `function`: function to be added
pub fn register_function<T: 'static>(function: Box<dyn StaticFunction>) {
    let mut registry = TYPE_REGISTRY.lock().unwrap();
    let short_name = type_shortname::<T>();

    // get type associated with this ctor (or create type entry)
    let type_info = registry.entry(short_name.clone()).or_insert_with(|| {
        Arc::new(TypeInfo {
            name: short_name,
            objtype: TypeId::of::<T>(),
            constructors: Vec::new(),
            methods: HashMap::new(),
            functions: HashMap::new()
        })
    });

    let key = function.name().to_string();
    Arc::make_mut(type_info).functions.insert(key, function);
}
