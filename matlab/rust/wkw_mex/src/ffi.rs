pub use std::os::raw::c_char;
pub use libc::{c_void, c_uchar, c_int, c_uint, c_double, size_t};

// HACK(amotta): There is no guarantee for this
#[allow(non_camel_case_types)]
type c_bool = bool;

pub enum MxArrayT {}
pub type MxArray = *const MxArrayT;
pub type MxArrayMut = *mut MxArrayT;

// HACK(amotta): This is only true for the new, 64bit API
pub type MwSize = size_t;

#[link(name = "mx")]
extern {
    // creation
    pub fn mxCreateNumericArray(
        ndim: size_t,
        dims: *const size_t,
        class_id: c_int,
        complex_flag: c_int
    ) -> MxArrayMut;
    pub fn mxMalloc(n: MwSize) -> *mut c_void;

    // access
    pub fn mxGetPr(pm: MxArray) -> *mut c_double;
    pub fn mxGetData(pm: MxArray) -> *mut c_void;
    pub fn mxArrayToUTF8String(pm: MxArray) -> *const c_char;
    pub fn mxGetNumberOfDimensions(pm: MxArray) -> size_t;
    pub fn mxGetDimensions(pm: MxArray) -> *const size_t;
    pub fn mxGetNumberOfElements(pm: MxArray) -> size_t;
    pub fn mxGetElementSize(pm: MxArray) -> size_t;

    // validation
    pub fn mxIsChar(pm: MxArray) -> c_bool;
    pub fn mxIsScalar(pm: MxArray) -> c_bool;
    pub fn mxIsDouble(pm: MxArray) -> c_bool;
    pub fn mxIsComplex(pm: MxArray) -> c_bool;
}

#[link(name = "mex")]
extern {
    pub fn mexErrMsgTxt(errormsg: *const c_uchar);
}
