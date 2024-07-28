
use reflect::{reflect_type, find_type};

struct Test1 {
    alpha: i32,
    beta: f64
}

#[reflect_type]
impl Test1 {
    fn new (a: i32) -> Self {
        return Test1 { alpha: a, beta: f64::from(a) * f64::from(a) };
    }
    fn new (a: i32, b: f64) -> Self {
        return Test1 { alpha: a, beta: b };
    }
}

#[test]
fn test_ctors1() {
    let args =&[Box::new(3i32), Box::new(3.1415926f64)];
    let itype = TypeInfo::find_type ("Test1").expect ("could not find type");

    //let rawobj = reflect_create("Test1", args).expect("failed to call ctor");
    //let obj = rawobj.downcast_ref::<Test1>().expect("faied to downcast to type");

    //assert_eq!(obj.alpha, 3);
    //assert_eq!(obj.beta, 3.1415926);
}
