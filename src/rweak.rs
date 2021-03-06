//! R Weak Reference type
//!
//!

use ::rdll::*;
use ::storage::*;
use ::traits::*;
use ::rtype::*;
use error::{RResult, rerror};
use error::REKind::{NotCompatible, Other};
use std::marker::PhantomData;
use std::any::*;

use symbol::*;
use util::*;

pub type RWeak<Obj> = RWeakM<Obj, Preserve>;


#[derive(Debug)]
pub struct RWeakM<Obj: Any, T: SEXPbucket> {
    data: T,
    obj: PhantomData<Obj>,
    tag: SEXP,
}


impl<Obj: Any, T: SEXPbucket> RWeakM<Obj, T> {
    pub fn key(&self) -> SEXP {
        unsafe { R_WeakRefKey(self.s()) }
    }
    pub fn value(&self) -> SEXP {
        unsafe { R_WeakRefValue(self.s()) }
    }
}

impl<Obj: Any, T: SEXPbucket> RNew for RWeakM<Obj, T> {
    fn rnew(x: SEXP) -> RResult<RWeakM<Obj, T>> {
        let tags: Symbol = c_str(format!("{:?}", TypeId::of::<Obj>()).as_ref()).into();
        unsafe {
            if RTYPEOF(x) != WEAKREFSXP {
                return rerror(NotCompatible(format!("expecting a {:?} pointer",
                                                    TypeId::of::<Obj>())
                                                .into()));
            }
            let res = RWeakM {
                data: T::new(x),
                obj: PhantomData,
                tag: tags.s(),
            };
            Ok(res)
        }
    }
}

// boil

gen_traits_rptr!(RWeakM);


pub struct RPtrParam<Obj: Any, T: SEXPbucket> {
    drop: bool,
    exit: bool,
    tag: SEXP,
    prot: SEXP,
    value: SEXP,
    obj: PhantomData<Obj>,
    t: PhantomData<T>,
    finalizer_func: ::std::option::Option<extern "C" fn(arg1: SEXP)>,
}

impl<Obj: Any, T: SEXPbucket> Default for RPtrParam<Obj, T> {
    fn default() -> RPtrParam<Obj, T> {
        unsafe {
            let sym: Symbol = c_str(format!("{:?}", TypeId::of::<Obj>()).as_ref()).into();
            RPtrParam {
                drop: true,
                exit: true,
                tag: sym.s(),
                prot: R_NilValue,
                value: R_NilValue,
                obj: PhantomData,
                t: PhantomData,
                finalizer_func: Some(finalizer_ptr::<Obj>),
            }
        }
    }
}


impl<Obj: Any, T: SEXPbucket> RPtrParam<Obj, T> {
    pub fn tag(mut self, tag_: SEXP) -> RPtrParam<Obj, T> {
        self.tag = tag_;
        self
    }
    pub fn prot(mut self, prot_: SEXP) -> RPtrParam<Obj, T> {
        self.prot = prot_;
        self
    }
    pub fn value(mut self, value_: SEXP) -> RPtrParam<Obj, T> {
        self.value = value_;
        self
    }
    pub fn drop(mut self, finalizer_: bool) -> RPtrParam<Obj, T> {
        self.drop = finalizer_;
        self
    }
    pub fn finalizer(mut self,
                     finalizer_: ::std::option::Option<extern "C" fn(arg1: SEXP)>)
                     -> RPtrParam<Obj, T> {
        self.finalizer_func = finalizer_;
        self
    }
    pub fn onexit(mut self, exit: bool) -> RPtrParam<Obj, T> {
        self.exit = exit;
        self
    }
    pub fn done(self, ptr: Box<Obj>) -> RWeakM<Obj, T>
        where T: SEXPbucket
    {
        let onexit: Rboolean;
        if self.exit {
            onexit = Rboolean::TRUE;
        } else {
            onexit = Rboolean::FALSE;
        }
        let extptr: SEXP;
        unsafe {
            extptr = R_MakeWeakRef(::std::mem::transmute(Box::into_raw(ptr)),
                                   self.value,
                                   R_NilValue,
                                   onexit);
        }

        let res = RWeakM {
            data: T::new(extptr),
            obj: PhantomData,
            tag: self.tag,
        };

        if self.drop {
            unsafe {
                R_RegisterCFinalizerEx(extptr, self.finalizer_func, onexit);
            }
        }
        res
    }
}

impl<Obj: Any, T: SEXPbucket> RWeakM<Obj, T> {
    pub fn init() -> RPtrParam<Obj, T> {
        RPtrParam::default()
    }
    pub fn new(ptr: Box<Obj>) -> RWeakM<Obj, T> {
        Self::init().done(ptr)
    }
    pub fn get(& mut self) -> RResult<&mut Obj> {
        unsafe {
            let res: *mut Obj = ::std::mem::transmute(R_ExternalPtrAddr(self.data.s()));
            if res.is_null() {
                return rerror(Other("external pointer is not valid".into()));
            }
            Ok(&mut *res)
        }
    }
    pub unsafe fn uget(&mut self) -> *mut Obj {
        ::std::mem::transmute(R_ExternalPtrAddr(self.data.s()))
    }
    //  An example release function.
    //  User should impl this themselves, because finalizer_ptr::<Obj> may change.
    //
    // 	pub fn release(self){
    // 		unsafe{
    // 				let res : * mut Obj= ::std::mem::transmute(R_ExternalPtrAddr(self.data.s()));
    // 				if res.is_null() != true {
    // 					finalizer_ptr::<Obj>(self.data.s());
    // 				}
    // 				R_ClearExternalPtr(self.data.s());
    // 		}
    //    }

    pub fn init_sexp(x: SEXP, tag: SEXP, prot: SEXP) -> RResult<RWeakM<Obj, T>> {
        let tags: Symbol = c_str(format!("{:?}", TypeId::of::<Obj>()).as_ref()).into();
        unsafe {
            if RTYPEOF(x) != WEAKREFSXP || tags.s() != R_ExternalPtrTag(x) {
                return rerror(NotCompatible(format!("expecting a {:?} pointer",
                                                    TypeId::of::<Obj>())
                                                .into()));
            }
            R_SetExternalPtrTag(x, tag);
            R_SetExternalPtrProtected(x, prot);
            let res = RWeakM {
                data: T::new(x),
                obj: PhantomData,
                tag: tags.s(),
            };
            Ok(res)
        }
    }

    pub fn get_tag(&self) -> SEXP {
        unsafe { R_ExternalPtrTag(self.data.s()) }
    }
    pub fn set_tag(&self, x: SEXP) {
        unsafe { R_SetExternalPtrTag(self.data.s(), x) };
    }
    pub fn get_prot(&self) -> SEXP {
        unsafe { R_ExternalPtrProtected(self.data.s()) }
    }
    pub fn set_prot(&self, x: SEXP) {
        unsafe { R_SetExternalPtrProtected(self.data.s(), x) };
    }
}

pub extern "C" fn finalizer_ptr<Obj: Any>(x: SEXP) {
    unsafe {
        if RTYPEOF(x) == WEAKREFSXP {
            Box::<Obj>::from_raw(::std::mem::transmute(R_ExternalPtrAddr(x)));
        }
    }
}
