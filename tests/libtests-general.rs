extern crate rustr;

pub use rustr::feature::engine::*;
pub use rustr::*;

#[test]
fn test_internals_one() {
    let mut re = unsafe { REngine::init().unwrap() };
    let max_fn: RFun = re.eval("max").unwrap();
    assert!(max_fn.is_function());
    let inp = intvec![1, 5, 10];
    let res: IntVec = max_fn.eval(&[&inp]).unwrap();
    let at = res.at(0).unwrap();
    assert_eq!(at, 10)
}

#[test]
fn test_rerror() {
    let mut re = unsafe { REngine::init().unwrap() };
    let res = re.eval("syntax error producing line;");
    match res {
        RError(x) => ,
        _ => {
            assert!(false)
        },
    }
}
