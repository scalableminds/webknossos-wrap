use ffi::*;

use std;
use std::ffi::CStr;
use std::slice;

pub type Result<T> = std::result::Result<T, String>;

pub fn as_nat(f: f64) -> Result<u64> {
    if f <= 0.0 {
        return Err("Input must be positive".to_string());
    }

    match f % 1.0 == 0.0 {
        true => Ok(f as u64),
        false => Err("Input must be an integer".to_string()),
    }
}

pub fn mx_array_to_str<'a>(pm: MxArray) -> Result<&'a str> {
    let pm_ptr = unsafe { mxArrayToUTF8String(pm) };

    if pm_ptr.is_null() {
        return Err("mxArrayToUTF8String returned null".to_string());
    }

    let pm_cstr = unsafe { CStr::from_ptr(pm_ptr) };

    match pm_cstr.to_str() {
        Ok(pm_str) => Ok(pm_str),
        Err(_) => Err("mxArray contains invalid UTF-8 data".to_string()),
    }
}

pub fn mx_array_to_f64_slice<'a>(pm: MxArray) -> Result<&'a [f64]> {
    unsafe {
        if !mxIsDouble(pm) {
            return Err("MxArray is not of class \"double\"".to_string());
        };
        if mxIsComplex(pm) {
            return Err("MxArray is complex".to_string());
        };
    }

    let pm_numel = unsafe { mxGetNumberOfElements(pm) };
    let pm_ptr = unsafe { mxGetPr(pm) };

    match pm_ptr.is_null() {
        true => Err("MxArray does not contain real values".to_string()),
        false => Ok(unsafe { slice::from_raw_parts(pm_ptr, pm_numel) }),
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

    unsafe { slice::from_raw_parts(dims, ndims as usize) }
}

pub fn create_numeric_array(
    dims: &[u64],
    class: MxClassId,
    complexity: MxComplexity,
) -> Result<MxArrayMut> {
    let arr = unsafe {
        mxCreateNumericArray(
            dims.len() as size_t,
            dims.as_ptr() as *const usize,
            class as c_int,
            complexity as c_int,
        )
    };

    match arr.is_null() {
        true => Err("Failed to create uninitialized numeric array".to_string()),
        false => Ok(arr),
    }
}

pub fn malloc(n: usize) -> Result<&'static mut [u8]> {
    let ptr = unsafe { mxMalloc(n as MwSize) } as *mut u8;

    match ptr.is_null() {
        true => Err("Failed to allocate memory".to_string()),
        false => Ok(unsafe { slice::from_raw_parts_mut(ptr, n) }),
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

pub fn copy_as_fortran_order(
    in_buf: &[u8],
    out_arr: MxArrayMut,
    shape: &[u64],
    type_size: usize,
) -> Result<()> {
    let total_elems: usize = shape.iter().product::<u64>() as usize;
    if in_buf.len() != total_elems * type_size {
        return Err(format!(
            "Length of input buffer does not match expected size {} != {}",
            in_buf.len(),
            total_elems,
        ));
    }

    let result = mx_array_mut_to_u8_slice_mut(out_arr)?;
    if result.len() != total_elems * type_size {
        return Err(format!(
            "Length of output array does not match expected size {} != {}",
            result.len(),
            total_elems,
        ));
    }

    // Compute F-order (column-major) strides
    let mut f_strides = vec![1u64; shape.len()];
    for i in 1..shape.len() {
        f_strides[i] = f_strides[i - 1] * shape[i - 1];
    }

    // Multi-dimensional index for C-order iteration
    let mut idx = vec![0u64; shape.len()];

    // Iterate over all elements in C-order (sequential read)
    for elem_idx in 0..total_elems {
        // Compute Fortran-order (column-major) offset
        let f_offset_elems: u64 = idx.iter().zip(&f_strides).map(|(&i, &s)| i * s).sum();
        let f_offset_bytes = f_offset_elems as usize * type_size;

        // Sequential read from in_buf
        let src_offset_bytes = elem_idx * type_size;
        let src_slice = &in_buf[src_offset_bytes..src_offset_bytes + type_size];

        // Scattered write to result
        result[f_offset_bytes..f_offset_bytes + type_size].copy_from_slice(src_slice);

        // Increment multi-dimensional index (C-order)
        for d in (0..shape.len()).rev() {
            idx[d] += 1;
            if idx[d] < shape[d] {
                break;
            } else if d > 0 {
                idx[d] = 0;
            }
        }
    }

    Ok(())
}

fn f64_slice_to_vec(buf: &[f64]) -> Result<Vec<u64>> {
    buf.iter()
        .map(|x| as_nat(*x).or(Err("Invalid value".to_string())))
        .collect()
}

pub fn mx_array_to_bbox(pm: MxArray, ndim: usize) -> Result<(Vec<u64>, Vec<u64>)> {
    let buf = mx_array_to_f64_slice(pm)?;

    // verify shape of array
    let input_arg_shape = mx_array_size_to_usize_slice(pm);
    if input_arg_shape != &[ndim, 2] {
        return Err(format!(
            "Bounding box has invalid shape. Needs to be [{}, 2]. Got {:?}.",
            ndim, input_arg_shape
        ));
    }

    let bbox_min_f64 = &buf[0..ndim];
    let bbox_max_f64 = &buf[ndim..(ndim * 2)];
    let bbox_min = f64_slice_to_vec(bbox_min_f64)
        .or(Err(format!("Invalid lower bound. Got {:?}.", bbox_min_f64)))?;
    let bbox_max = f64_slice_to_vec(bbox_max_f64)
        .or(Err(format!("Invalid upper bound. Got {:?}.", bbox_max_f64)))?;

    if bbox_min
        .iter()
        .zip(bbox_max.iter())
        .any(|(min_x, max_x)| min_x >= max_x)
    {
        return Err(format!(
            "Bounding box has invalid shape. Got min={:?}, max={:?}.",
            bbox_min, bbox_max
        ));
    }

    let bbox_shape: Vec<u64> = bbox_min
        .iter()
        .zip(bbox_max.iter())
        .map(|(min_x, max_x)| max_x - min_x)
        .collect();

    let bbox_min = bbox_min.iter().map(|x| x - 1).collect();

    if bbox_shape.iter().any(|x| *x < 1) {
        return Err(format!(
            "Bounding box has invalid shape. Got {:?}.",
            bbox_shape
        ));
    }

    Ok((bbox_min, bbox_shape))
}
