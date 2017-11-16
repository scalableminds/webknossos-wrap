extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;

use std::slice;
use std::path::Path;

mex_function!(nlhs, lhs, nrhs, rhs, {
    let rhs = match nrhs == 2 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments")
    };

    let lhs = match nlhs == 1 {
        true => slice::from_raw_parts_mut(lhs, nlhs as usize),
        false => return Err("Invalid number of output arguments")
    };

    let wkw_path = mx_array_to_str(rhs[0])?;
    let dataset_path = Path::new(wkw_path);
    let dataset = wkwrap::Dataset::new(dataset_path)?;

    let num_channels = dataset.header().num_channels();
    let is_multi_channel = num_channels > 0;

    // build shape
    let bbox = mx_array_to_wkwrap_box(rhs[1])?;
    let shape = bbox.width();
    
    let shape_arr = [num_channels, shape.x as usize, shape.y as usize, shape.z as usize];
    let shape_slice = if is_multi_channel { &shape_arr[0..] } else { &shape_arr[1..] };

    // prepare allocation
    let voxel_type = dataset.header().voxel_type;
    let class = voxel_type_to_mx_class_id(voxel_type);

    // read data
    let arr = create_numeric_array(shape_slice, class, MxComplexity::Real)?;
    let mut mat = mx_array_mut_to_wkwrap_mat(is_multi_channel, arr)?;
    dataset.read_mat(bbox.min(), &mut mat)?;

    // set output
    lhs[0] = arr;

    Ok(())
});
