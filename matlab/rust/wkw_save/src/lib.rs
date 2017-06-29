extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;

use std::slice;
use std::path::Path;

mex_function!(_nlhs, _lhs, nrhs, rhs, {
    let rhs = match nrhs == 3 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments")
    };

    let wkw_path = mx_array_to_str(rhs[0])?;
    let pos = mx_array_to_wkwrap_vec(rhs[1])? - 1;
    let data = mx_array_to_wkwrap_mat(rhs[2])?;

    let dataset_path = Path::new(wkw_path);
    let dataset = wkwrap::Dataset::new(dataset_path)?;
    dataset.write_mat(pos, &data)?;

    Ok(())
});
