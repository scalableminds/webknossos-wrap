use ::ffi::*;

use std;
use std::slice;
use std::ffi::CStr;

pub type Result<T> = std::result::Result<T, String>;

pub fn as_nat(f: f64) -> Result<u64> {
    if f <= 0.0 {
        return Err("Input must be positive".to_string())
    }

    match f % 1.0 == 0.0 {
        true => Ok(f as u64),
        false => Err("Input must be an integer".to_string())
    }
}

pub fn as_log2(f: f64) -> Result<u8> {
    let i = as_nat(f)?;

    match i & (i - 1) == 0 {
        true => Ok(i.trailing_zeros() as u8),
        false => Err("Input must be a power of two".to_string())
    }
}

pub fn str_slice_to_mx_class_id(class_id: &str) -> Result<MxClassId> {
    match class_id {
        "uint8"  => Ok(MxClassId::Uint8),
        "uint16" => Ok(MxClassId::Uint16),
        "uint32" => Ok(MxClassId::Uint32),
        "uint64" => Ok(MxClassId::Uint64),
        "single" => Ok(MxClassId::Single),
        "double" => Ok(MxClassId::Double),
        "int8"   => Ok(MxClassId::Int8),
        "int16"  => Ok(MxClassId::Int16),
        "int32"  => Ok(MxClassId::Int32),
        "int64"  => Ok(MxClassId::Int64),
        _        => Err("Unknown MxClassId name".to_string())
    }
}

pub fn mx_array_to_str<'a>(pm: MxArray) -> Result<&'a str> {
    let pm_ptr = unsafe { mxArrayToUTF8String(pm) };

    if pm_ptr.is_null() {
        return Err("mxArrayToUTF8String returned null".to_string())
    }

    let pm_cstr = unsafe { CStr::from_ptr(pm_ptr) };

    match pm_cstr.to_str() {
        Ok(pm_str) => Ok(pm_str),
        Err(_) => Err("mxArray contains invalid UTF-8 data".to_string())
    }
}

pub fn mx_array_to_f64_slice<'a>(pm: MxArray) -> Result<&'a [f64]> {
    unsafe {
        if !mxIsDouble(pm) { return Err("MxArray is not of class \"double\"".to_string()) };
        if mxIsComplex(pm) { return Err("MxArray is complex".to_string()) };
    }

    let pm_numel = unsafe { mxGetNumberOfElements(pm) };
    let pm_ptr = unsafe { mxGetPr(pm) };

    match pm_ptr.is_null() {
        true => Err("MxArray does not contain real values".to_string()),
        false => Ok(unsafe { slice::from_raw_parts(pm_ptr, pm_numel) })
    }
}

pub fn mx_array_to_f64(pm: MxArray) -> Result<f64> {
    let pm_slice = mx_array_to_f64_slice(pm)?;

    match pm_slice.len() {
        1 => Ok(pm_slice[0]),
        _ => Err("MxArray contains an invalid number of doubles".to_string())
    }
}

pub fn mx_array_to_u8_slice<'a>(pm: MxArray) -> Result<&'a [u8]> {
    let numel = unsafe { mxGetNumberOfElements(pm) };
    let elem_size = unsafe { mxGetElementSize(pm) };
    let data = unsafe { mxGetData(pm) } as *const u8;

    if elem_size == 0 {
        Err("Failed to determine element size".to_string())
    } else if data.is_null() {
        Err("Data pointer is null".to_string())
    } else {
        Ok(unsafe { slice::from_raw_parts(data, numel * elem_size) })
    }
}

pub fn mx_array_mut_to_u8_slice_mut<'a>(pm: MxArrayMut) -> Result<&'a mut [u8]> {
    let numel = unsafe { mxGetNumberOfElements(pm) };
    let elem_size = unsafe { mxGetElementSize(pm) };
    let data = unsafe { mxGetData(pm) } as *mut u8;

    if elem_size == 0 {
        Err("Failed to determine element size".to_string())
    } else if data.is_null() {
        Err("Data pointer is null".to_string())
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
        true => Err("Failed to create uninitialized numeric array".to_string()),
        false => Ok(arr)
    }
}

pub fn malloc(n: usize) -> Result<&'static mut [u8]> {
    let ptr = unsafe { mxMalloc(n as MwSize) } as *mut u8;

    match ptr.is_null() {
        true => Err("Failed to allocate memory".to_string()),
        false => Ok(unsafe { slice::from_raw_parts_mut(ptr, n) })
    }
}

pub fn die(msg: &str) {
    let bytes = msg.as_bytes();
    let len = bytes.len();

    // build zero-terminated string
    let buf = malloc(len + 1).unwrap();
    buf[..len].copy_from_slice(bytes);
    buf[len] = 0;

    // die
    unsafe { mexErrMsgTxt(buf.as_ptr()) }
}
