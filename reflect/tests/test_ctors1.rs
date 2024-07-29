
use reflect::{TypeInfo, Constructor, Method};
use reflect_macros::reflect_type;
use std::any::Any;


struct Test1 {
    alpha: i32,
    beta: f64
}

#[reflect_type]
impl Test1 {
    fn create1 (a: i32) -> Self {
        return Test1 { alpha: a, beta: f64::from(a) * f64::from(a) };
    }
    fn create2 (a: i32, b: f64) -> Self {
        return Test1 { alpha: a, beta: b };
    }
}


#[test]
fn test_ctors1() {
    let args = &[
        Box::new(3i32) as Box<dyn Any>,
        Box::new(3.1415926f64) as Box<dyn Any>
    ];
    let itype = TypeInfo::find_type(&String::from("Test1")).expect("could not find type");

    //let rawobj = reflect_create("Test1", args).expect("failed to call ctor");
    //let obj = rawobj.downcast_ref::<Test1>().expect("faied to downcast to type");

    //assert_eq!(obj.alpha, 3);
    //assert_eq!(obj.beta, 3.1415926);
}
