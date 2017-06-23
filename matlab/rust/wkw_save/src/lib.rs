extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;

use std::slice;
use std::path::Path;

#[no_mangle]
mex_function!(nlhs, plhs, nrhs, prhs, {
    if nrhs != 3 {
        mexErrMsgTxt("Invalid number of input arguments!\0".as_ptr());
    }

    if nlhs != 0 {
        mexErrMsgTxt("Invalid number of output arguments!\0".as_ptr());
    }

    let rhs = slice::from_raw_parts(prhs, 3);
    let wkw_path = mx_array_to_str(rhs[0]).unwrap();
    let pos = mx_array_to_wkwrap_vec(rhs[1]).unwrap();
    let data = mx_array_to_wkwrap_mat(rhs[2]).unwrap();

    let dataset_path = Path::new(wkw_path);
    let dataset = wkwrap::Dataset::new(dataset_path).unwrap();
    dataset.write_mat(pos, &data).unwrap();
});
