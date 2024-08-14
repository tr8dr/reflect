
use reflect::{TypeInfo, Constructor, Method };
use reflect_macros::reflect_impl;
use std::any::Any;


struct Test1 {
    alpha: i32,
    beta: f64
}

#[reflect_impl]
impl Test1 {
    fn create1 (a: i32) -> Self {
        return Test1 { alpha: a, beta: f64::from(a) * f64::from(a) };
    }
    fn create2 (a: i32, b: f64) -> Self {
        return Test1 { alpha: a, beta: b };
    }

    fn f(&self, x: i32) -> i32 {
        return x * self.alpha;
    }

    fn g(&self, vec: &[f64]) -> f64 {
        let mut cum = 0.0;
        for v in vec {
            cum += *v;
        }
        cum
    }
}


#[test]
fn test_ctors1() {
    let args = vec![
        Box::new(3i32) as Box<dyn Any>,
        Box::new(3.1415926f64) as Box<dyn Any>
    ];
    let itype = TypeInfo::find_type(&String::from("Test1")).expect("could not find type");

    // create object
    let rawobj = itype.create(&args).expect("failed to call ctor");
    // downcast object
    let obj = rawobj.downcast_ref::<Test1>().expect("faied to downcast to type");

    assert_eq!(obj.alpha, 3);
    assert_eq!(obj.beta, 3.1415926);
}


#[test]
fn test_method1() {
    let args_ctor = vec![
        Box::new(3i32) as Box<dyn Any>,
        Box::new(3.1415926f64) as Box<dyn Any>
    ];
    let args_fun = vec![
        Box::new(4i32) as Box<dyn Any>
    ];

    let itype = TypeInfo::find_type(&String::from("Test1")).expect("could not find type");

    // create object
    let rawobj = itype.create(&args_ctor).expect("failed to call ctor");

    // call function on object
    let result = match itype.call(&rawobj, "f", &args_fun) {
        Ok(v) => v.downcast::<i32>().unwrap(),
        Err(_) => panic!("could not call function")
    };

    assert_eq!(*result, 12);
}


#[test]
fn test_method2() {
    let args_ctor = vec![
        Box::new(3i32) as Box<dyn Any>,
    ];
    let args_fun = vec![
        Box::new(vec![3.1, 4.2, 5.1]) as Box<dyn Any>
    ];

    let itype = TypeInfo::find_type(&String::from("Test1")).expect("could not find type");

    // create object
    let rawobj = itype.create(&args_ctor).expect("failed to call ctor");

    // call function on object
    let result = match itype.call(&rawobj, "g", &args_fun) {
        Ok(v) => v.downcast::<f64>().unwrap(),
        Err(e) => panic!("{}", e)
    };

    assert_eq!(*result, 12.4);
}
