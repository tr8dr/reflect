
use std::any::{TypeId};
use std::any::type_name;

use crate::core::{Function};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{RwLock,Arc};
use std::any::Any;
use std::str::FromStr;

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

            let vi32 = TypeId::of::<Vec<i32>>();
            let vi64 = TypeId::of::<Vec<i64>>();
            let vf64 = TypeId::of::<Vec<f64>>();

            let si32 = TypeId::of::<&[i32]>();
            let si64 = TypeId::of::<&[i64]>();
            let sf64 = TypeId::of::<&[f64]>();

            // i32 conversions
            add (ti32, ti32, Conversions::EQUIVALENT,
                |x| { to::<i32,i32>(x) } );
            add (ti32, ti64, 100,
                |x| { to::<i32,i64>(x) });
            add (ti32, tu32, 150,
                |x| { to::<i32,u32>(x) });
            add (ti32, tu64, 100,
                |x| { to::<i32,u64>(x) });
            add (ti32, tf64, 150,
                |x| { to::<i32,f64>(x) });

            // u32 conversions
            add (tu32, tu32, Conversions::EQUIVALENT,
                |x| { to::<u32,u32>(x) });
            add (tu32, ti32, 150,
                |x| { to::<u32,i32>(x) });
            add (tu32, ti64, 150,
                |x| { to::<u32,i64>(x) });
            add (tu32, tu64, 150,
                |x| { to::<u32,u64>(x) });
            add (tu32, tf64, 150,
                |x| { to::<u32,f64>(x) });

            // i64 conversions
            add (ti64, ti64, Conversions::EQUIVALENT,
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
            add (tu64, tu64, Conversions::EQUIVALENT,
                |x| { to::<u64,u64>(x) });
            add (tu64, ti32, 100,
                |x| { to::<u64,i32>(x) });
            add (tu64, tu32, 100,
                |x| { to::<u64,u32>(x) });
            add (tu64, ti64, 150,
                |x| { to::<u64,i64>(x) });
            add (tu64, tf64, 100,
                |x| { Some(Box::new(raw::<u64>(x) as f64) as Box<dyn Any>) });

            // f64 conversions
            add (tf64, tf64, Conversions::EQUIVALENT,
                |x| { to::<f64,f64>(x) });
            add (tf64, ti32, 150,
                |x| { Some(Box::new(raw::<f64>(x).round() as i32) as Box<dyn Any>) });
            add (tf64, tu32, 100,
                |x| { Some(Box::new(raw::<f64>(x).round() as u32) as Box<dyn Any>) });
            add (tf64, tu64, 150,
                |x| { Some(Box::new(raw::<f64>(x).round() as u64) as Box<dyn Any>) });
            add (tf64, ti64, 150,
                |x| { Some(Box::new(raw::<f64>(x).round() as i64) as Box<dyn Any>) });

            // string conversions
            add (tstr, tstr, Conversions::EQUIVALENT,
                |x| { Some(Box::new(raw::<&String>(x)) as Box<dyn Any>) });
            add (tstr, ti32, 50,
                |x| { try_parse::<i32>(x) });
            add (tstr, tu32, 50,
                |x| { try_parse::<u32>(x) });
            add (tstr, ti64, 50,
                |x| { try_parse::<i64>(x) });
            add (tstr, tu64, 50,
                |x| { try_parse::<u64>(x) });
            add (tstr, tf64, 50,
                |x| { try_parse::<f64>(x) });

            // vector conversions
            add (vi32, si32, Conversions::EQUIVALENT,
                |x| { convert_vec::<i32,i32>(x) });
            add (vi32, sf64, Conversions::EQUIVALENT,
                |x| { convert_vec::<i32,f64>(x) });
            add (vi64, si64, Conversions::EQUIVALENT,
                |x| { convert_vec::<i64,i64>(x) });
            add (vf64, sf64, Conversions::EQUIVALENT,
                |x| { convert_vec::<f64,f64>(x) });
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
    const EQUIVALENT: i32 = 200;

    /// Indicate whether this conversion pairing is T -> T or equivalent
    pub fn is_equivalent (&self) -> bool {
        self.score == Conversions::EQUIVALENT
    }

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

    /// Score a given argument vector versus target parameter types
    /// - higher score implies a better fit
    ///
    /// # Arguments
    /// * `tartget`: function parameter types
    /// * `args`: incoming argument vector for function
    ///
    /// # Returns
    /// * score for given argument set.  Higher positive value -> better fit and lower implies
    ///   worse fit.  A negative score implies no fit at all
    pub fn score (target: &[TypeId], args: &[Box<dyn Any>]) -> i32 {
        // if # of args and parameters don't match punt
        if target.len() != args.len() {
            return -200;
        }

        // otherwise score parameters
        let mut score = 0;
        for (to_arg, from_arg) in target.iter().zip(args) {
            let arg_type = (**from_arg).type_id();
            match Conversions::find(arg_type, *to_arg) {
                Some(conversion) => {
                    score += conversion.score;
                }
                None => {
                    score = -100;
                    break
                }
            }
        }
        score
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

            // evaluate score of given arguments relative to argument types of candidate
            let score = Self::score(cargs, args);

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
    /// * `parameters`: target function parameter types
    /// * `args`: incoming argv to be converted
    ///
    /// # Returns
    /// * converted arguments or None if failed
    pub fn convert_argv (parameters: &[TypeId], args: &[Box<dyn Any>]) -> Option<Vec<Box<dyn Any>>> {
        // check target args vs provided args
        if parameters.len() != args.len() {
            return None;
        }

        let mut newargs: Vec<Box<dyn Any>> = Vec::new();
        for (to_type, from_arg) in parameters.iter().zip(args) {
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


// Copy vector from type T to type R
fn convert_vec<T, R>(boxed: &Box<dyn Any>) -> Option<Box<dyn Any>>
where
    T: 'static + Clone,
    R: 'static + TryFrom<T>,
    <R as TryFrom<T>>::Error: std::fmt::Debug,
{
    boxed.downcast_ref::<Vec<T>>().map(|vec| {
        let converted: Vec<R> = vec.iter()
            .filter_map(|item| R::try_from(item.clone()).ok())
            .collect();
        Box::new(converted) as Box<dyn Any>
    })
}

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

// Parse a string to a primitive type
fn try_parse<T: 'static + Copy + FromStr> (v: &Box<dyn Any>) -> Option<Box<dyn Any>> {
    let raw: &String = *v.downcast_ref::<&String>().unwrap();
    match (*raw).parse::<T>() {
        Ok(v) => Some(Box::new(v) as Box<dyn Any>),
        Err(_) => None
    }
}