extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;

use std::slice;
use std::path::Path;

mex_function!(_nlhs, _lhs, nrhs, rhs, {
    let rhs = match nrhs == 3 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments".to_string())
    };

    // path to root
    let wkw_path = mx_array_to_str(rhs[0])?;
    let dataset_path = Path::new(wkw_path);
    let dataset = wkwrap::Dataset::new(dataset_path)?;

    // offset
    let off = mx_array_to_wkwrap_vec(rhs[1])? - 1;

    // data
    let is_multi_channel = dataset.header().is_multi_channel();
    let data = mx_array_to_wkwrap_mat(is_multi_channel, rhs[2])?;

    dataset.write_mat(off, &data)?;

    Ok(())
});
