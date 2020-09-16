extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;

use std::slice;
use std::path::Path;

mex_function!(_nlhs, _lhs, nrhs, rhs, {
    let rhs = match nrhs == 2 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments".to_string())
    };

    let src_path = Path::new(mx_array_to_str(rhs[0])?);
    let dst_path = Path::new(mx_array_to_str(rhs[1])?);

    let mut file = wkwrap::File::open(&src_path)?;
    file.compress(&dst_path)
});
