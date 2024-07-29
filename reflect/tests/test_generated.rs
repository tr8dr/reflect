
use reflect::{TypeInfo, Constructor, Method };
use reflect_macros::reflect_type;
use std::any::Any;


struct Test2 {
    alpha: i32,
    beta: f64
}

#[reflect_type]
impl Test2 {
    fn create1 (a: i32) -> Self {
        return Test1 { alpha: a, beta: f64::from(a) * f64::from(a) };
    }
    fn create2 (a: i32, b: f64) -> Self {
        return Test1 { alpha: a, beta: b };
    }
}


#[derive(Clone)]
struct create1Method {
    name : String,
}
impl Callable for create1Method {
    fn call(& self, args : & [Box < dyn std :: any :: Any >]) -> Result < Box     < dyn std :: any :: Any > , String >  {
        let a = match args.get(0usize).and_then(| arg | arg.downcast_ref :: < i32 > ())  {
            Some(value) => value,
            None => return Err(format! ("Invalid argument type for parameter {}", 0usize)),
         };
         let result = Test1 :: create1(a); Ok(Box :: new(result))
    }
    fn arg_types(& self) -> & [std :: any :: TypeId] {
        static ARG_TYPES : & [std :: any :: TypeId] = &[std :: any :: TypeId :: of :: < i32 > ()];
        ARG_TYPES
    }
    fn return_type(& self) -> std :: any :: TypeId {
        std :: any :: TypeId :: of :: < Self > ()
    }
}

impl Method for create1Method {
    fn name(& self) -> & String {
        & self.name
    }

    fn clone_box(& self) -> Box <     dyn Method > {
        Box :: new(self.clone())
    }
}

static _REGISTER: () = {
    register_method::<Test1>(Box::new(create1Method { name: stringify!(create1).to_string(), }));
};

#[test]
fn test_generated() {
    let args = &[
        Box::new(3i32) as Box<dyn Any>,
        Box::new(3.1415926f64) as Box<dyn Any>
    ];
    let itype = TypeInfo::find_type(&String::from("Test2")).expect("could not find type");
}
