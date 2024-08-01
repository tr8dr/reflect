
use std::any::{TypeId};
use std::any::type_name;

use crate::core::{Function};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{RwLock,Arc};
use std::any::Any;


// Conversion function type
type ConversionFn = fn(&dyn std::any::Any) -> Box<dyn std::any::Any>;

// Type conversions map
lazy_static! {
    static ref CONVERSIONS: RwLock<HashMap<(TypeId,TypeId),Arc<Conversion>>> = RwLock::new(HashMap::new());
}


/// Type conversion record
/// - note that we require a score so can rank possible alternative conversions; A
///   score of 100 would mean that has full conversion weight and a lower score
///   would make the conversion less likely to be picked
///
/// - for a group of arguments requiring conversion, the function with the highest score
///   relative to the supplied arguments would be selected
struct Conversion {
    from_type: TypeId,
    to_type: TypeId,
    score: i32,
    conversion: ConversionFn,
}

impl Conversion {
    /// Add a type conversion
    /// - note that we require a score so can rank possible alternative conversions; A
    ///   score of 100 would mean that has full conversion weight and a lower score
    ///   would make the conversion less likely to be picked
    ///
    /// - for a group of arguments requiring conversion, the function with the highest score
    ///   relative to the supplied arguments would be selected
    ///
    /// # Arguments
    /// * `from`: type to convert from
    /// * `to`: type to convert to
    /// * `score`: score for this conversion, score of 100 is best and score of 0 is worst
    /// * `convert`: conversion function, converting from `from` type to `to` type
    fn add (from: TypeId, to: TypeId, score: i32, convert: ConversionFn) {
        let conversion = Conversion {
            from_type: from,
            to_type: to,
            score: score,
            conversion: convert };

        // get writer handle to conversions
        let mut map = CONVERSIONS.write().unwrap();
        // add conversion
        map.insert ((from, to), Arc::new(conversion));
    }

    /// Find a conversion between `from` and `to`
    ///
    /// # Arguments
    /// * `from`: type to convert from
    /// * `to`: type to convert to
    ///
    /// # Returns
    /// * conversion or None
    fn find (from: TypeId, to: TypeId) -> Option<Arc<Conversion>> {
        let mut map = CONVERSIONS.read().unwrap();
        map.get(&(from,to)).cloned()
    }

    /// Find best matched ctor based on arguments
    /// - note that this method should only be used if the candidate list has been reduced to
    ///   those candidates with the appropriate name or for ctors, where the name is not
    ///   important
    ///
    /// # Arguments
    /// * `candidates`: list of candidate functions (ctors, methods, static methods)
    /// * `args`: argument list
    ///
    /// # Returns
    /// * best function or None if no convertible matches
    fn find_best_match<T: ?Sized + Function> (candidates: &[Box<T>], args: &[Box<dyn Any>]) -> Option<T> {
        
    }


}



