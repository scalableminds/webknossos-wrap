use ::ffi::*;

use std;
use std::slice;
use std::ffi::CStr;

pub type Result<T> = std::result::Result<T, &'static str>;

pub fn mx_array_to_str<'a>(pm: MxArray) -> Result<&'a str> {
    let pm_ptr = unsafe { mxArrayToUTF8String(pm) };

    if pm_ptr.is_null() {
        return Err("mxArrayToUTF8String returned null")
    }

    let pm_cstr = unsafe { CStr::from_ptr(pm_ptr) };

    match pm_cstr.to_str() {
        Ok(pm_str) => Ok(pm_str),
        Err(_) => Err("mxArray contains invalid UTF-8 data")
    }
}

pub fn mx_array_to_f64_slice<'a>(pm: MxArray) -> Result<&'a [f64]> {
    unsafe {
        if !mxIsDouble(pm) { return Err("MxArray is not of class \"double\"") };
        if mxIsComplex(pm) { return Err("MxArray is complex") };
    }

    let pm_numel = unsafe { mxGetNumberOfElements(pm) };
    let pm_ptr = unsafe { mxGetPr(pm) };

    match pm_ptr.is_null() {
        true => Err("MxArray does not contain real values"),
        false => Ok(unsafe { slice::from_raw_parts(pm_ptr, pm_numel) })
    }
}

pub fn mx_array_to_f64(pm: MxArray) -> Result<f64> {
    let pm_slice = mx_array_to_f64_slice(pm)?;

    match pm_slice.len() {
        1 => Ok(pm_slice[0]),
        _ => Err("MxArray contains an invalid number of doubles")
    }
}

pub fn mx_array_to_u8_slice<'a>(pm: MxArray) -> Result<&'a [u8]> {
    let numel = unsafe { mxGetNumberOfElements(pm) };
    let elem_size = unsafe { mxGetElementSize(pm) };
    let data = unsafe { mxGetData(pm) } as *const u8;

    if elem_size == 0 {
        Err("Failed to determine element size")
    } else if data.is_null() {
        Err("Data pointer is null")
    } else {
        Ok(unsafe { slice::from_raw_parts(data, numel * elem_size) })
    }
}

pub fn mx_array_mut_to_u8_slice_mut<'a>(pm: MxArrayMut) -> Result<&'a mut [u8]> {
    let numel = unsafe { mxGetNumberOfElements(pm) };
    let elem_size = unsafe { mxGetElementSize(pm) };
    let data = unsafe { mxGetData(pm) } as *mut u8;

    if elem_size == 0 {
        Err("Failed to determine element size")
    } else if data.is_null() {
        Err("Data pointer is null")
    } else {
        Ok(unsafe { slice::from_raw_parts_mut(data, numel * elem_size) })
    }
}

pub fn mx_array_size_to_usize_slice<'a>(pm: MxArray) -> &'a [usize] {
    let ndims = unsafe { mxGetNumberOfDimensions(pm) };
    let dims = unsafe { mxGetDimensions(pm) };

    unsafe {
        slice::from_raw_parts(dims, ndims as usize)
    }
}

pub fn create_numeric_array(
    dims: &[usize],
    class: MxClassId,
    complexity: MxComplexity
) -> Result<MxArrayMut> {
    let arr = unsafe {
        mxCreateNumericArray(
            dims.len() as size_t, dims.as_ptr(),
            class as c_int, complexity as c_int)
    };

    match arr.is_null() {
        true => Err("Failed to create uninitialized numeric array"),
        false => Ok(arr)
    }
}

pub fn malloc(n: usize) -> Result<&'static mut [u8]> {
    let ptr = unsafe { mxMalloc(n as MwSize) } as *mut u8;

    match ptr.is_null() {
        true => Err("Failed to allocate memory"),
        false => Ok(unsafe { slice::from_raw_parts_mut(ptr, n) })
    }
}

pub fn die(msg: &str) {
    let bytes = msg.as_bytes();
    let len = bytes.len();

    // build zero-terminated string
    let mut buf = malloc(len + 1).unwrap();
    buf[..len].copy_from_slice(bytes);
    buf[len] = 0;

    // die
    unsafe { mexErrMsgTxt(buf.as_ptr()) }
}
