
use std::any::{TypeId};
use std::any::type_name;

use crate::core::{Function};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{RwLock,Arc};
use std::any::Any;


// Conversion function type
type ConversionFn = fn(&Box<dyn Any>) -> Option<Box<dyn Any>>;


// Type conversions map
lazy_static! {
    static ref CONVERSIONS: RwLock<HashMap<(TypeId,TypeId),Arc<Conversions>>> = {
        let rawmap = RwLock::new(HashMap::<(TypeId,TypeId),Arc<Conversions>>::new());

        {
            let mut m = rawmap.write().unwrap();

            let mut add = |t1: TypeId, t2: TypeId, score: i32, f: ConversionFn| {
                m.insert((t1,t2), Arc::new(Conversions { score: score, convert: f}));
            };

            let ti32 = TypeId::of::<i32>();
            let tu32 = TypeId::of::<u32>();
            let ti64 = TypeId::of::<i64>();
            let tu64 = TypeId::of::<u64>();

            let tf64 = TypeId::of::<f64>();

            let tstr = TypeId::of::<String>();

            // i32 conversions
            add (ti32, ti32, 200,
                |x| { to::<i32,i32>(x) } );
            add (ti32, ti64, 150,
                |x| { to::<i32,i64>(x) });
            add (ti32, tu32, 200,
                |x| { to::<i32,u32>(x) });
            add (ti32, tu64, 150,
                |x| { to::<i32,u64>(x) });
            add (ti32, tf64, 200,
                |x| { to::<i32,f64>(x) });

            // u32 conversions
            add (tu32, tu32, 200,
                |x| { to::<u32,u32>(x) });
            add (tu32, ti32, 200,
                |x| { to::<u32,i32>(x) });
            add (tu32, ti64, 200,
                |x| { to::<u32,i64>(x) });
            add (tu32, tu64, 200,
                |x| { to::<u32,u64>(x) });
            add (tu32, tf64, 200,
                |x| { to::<u32,f64>(x) });

            // i64 conversions
            add (ti64, ti64, 200,
                |x| { to::<i64,i64>(x) });
            add (ti64, ti32, 100,
                |x| { to::<i64,i32>(x) });
            add (ti64, tu32, 100,
                |x| { to::<i64,u32>(x) });
            add (ti64, tu64, 150,
                |x| { to::<i64,u64>(x) });
            add (ti64, tf64, 100,
                |x| { to::<i64,i32>(x) });

            // u64 conversions
            add (tu64, tu64, 200,
                |x| { to::<u64,u64>(x) });
            add (tu64, ti32, 100,
                |x| { to::<u64,i32>(x) });
            add (tu64, tu32, 100,
                |x| { to::<u64,u32>(x) });
            add (tu64, ti64, 200,
                |x| { to::<u64,i64>(x) });
            add (tu64, tf64, 100,
                |x| { Some(Box::new(raw::<u64>(x) as f64) as Box<dyn Any>) });
        }
        rawmap
    };
}


/// Type conversion record
/// - note that we require a score so can rank possible alternative conversions; A
///   score of 200 would mean that has full conversion weight and a lower score
///   would make the conversion less likely to be picked
///
/// - for a group of arguments requiring conversion, the function with the highest score
///   relative to the supplied arguments would be selected
pub struct Conversions {
    score: i32,
    convert: ConversionFn,
}

impl Conversions {
    /// Add a type conversion
    /// - note that we require a score so can rank possible alternative conversions; A
    ///   score of 200 would mean that has full conversion weight and a lower score
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
    pub fn add (from: TypeId, to: TypeId, score: i32, convert: ConversionFn) {
        let conversion = Conversions {
            score: score,
            convert: convert };

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
    pub fn find (from: TypeId, to: TypeId) -> Option<Arc<Conversions>> {
        let map = CONVERSIONS.read().unwrap();
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
    pub fn find_best_match<'a, T: ?Sized + Function> (candidates: &'a [Box<T>], args: &[Box<dyn Any>]) -> Option<&'a T> {
        // nothing to do if no candidates provided
        if candidates.len() == 0 {
            return None
        }

        let mut best_candidate = &candidates[0];
        let mut best_score = -100;

        for candidate in candidates {
            let cargs: &[TypeId] = candidate.arg_types();
            if cargs.len() != args.len() { continue }

            // evaluate score of given arguments relative to argument types of candidate
            let mut score = 0;
            for (to_arg, from_arg) in cargs.iter().zip(args) {
                match Conversions::find(from_arg.type_id(), *to_arg) {
                    Some(conversion) => {
                        score += conversion.score;
                    }
                    None => {
                        score = -100;
                        break
                    }
                }
            }

            if score > best_score {
                best_score = score;
                best_candidate = candidate;
            }
        }

        return if best_score > 0 {
            Some(best_candidate)
        } else {
            None
        }
    }

    /// Convert incoming argument vector to be compatible with target function arguments
    ///
    /// # Arguments
    /// * `function`: the
    pub fn convert_argv<T: ?Sized + Function> (function: &Box<T>, args: &[Box<dyn Any>]) -> Option<Vec<Box<dyn Any>>> {
        // check target args vs provided args
        let fun_types = function.arg_types();
        if fun_types.len() != args.len() {
            return None;
        }

        let mut newargs: Vec<Box<dyn Any>> = Vec::new();
        for (to_type, from_arg) in fun_types.iter().zip(args) {
            match Conversions::find(from_arg.type_id(), *to_type) {
                Some(conversion) => {
                    let cfun = conversion.convert;
                    match cfun(from_arg) {
                        Some(v) => newargs.push(v),
                        None => return None
                    }
                }
                None => {
                    return None
                }
            }
        }

        Some(newargs)
    }
}


//
// Special conversions
//


// Conversion for boxed primitive types to another type
fn to<T: 'static,R: 'static> (v: &Box<dyn Any>) -> Option<Box<dyn Any>>  where T: Copy, R: TryFrom<T> {
    let r: Option<R> = v.downcast_ref::<T>().and_then(|value| { (*value).try_into().ok() });
    match r {
        Some(x) => Some(Box::new(x) as Box<dyn Any>),
        None => None
    }
}


// Get raw underlying value
fn raw<T: 'static> (v: &Box<dyn Any>) -> T  where T: Copy {
    *v.downcast_ref::<T>().unwrap()
}
