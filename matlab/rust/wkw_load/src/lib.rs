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

    let mut lhs = match nlhs == 1 {
        true => slice::from_raw_parts_mut(lhs, nlhs as usize),
        false => return Err("Invalid number of output arguments")
    };

    let wkw_path = mx_array_to_str(rhs[0])?;
    let bbox = mx_array_to_wkwrap_box(rhs[1])?;

    let dataset_path = Path::new(wkw_path);
    let dataset = wkwrap::Dataset::new(dataset_path)?;

    // prepare allocation
    let shape = bbox.width();
    let voxel_size = dataset.header().voxel_size as usize;
    let voxel_type = dataset.header().voxel_type;
    let voxel_type_size = voxel_type.size();

    let num_channels = match voxel_size % voxel_type_size == 0 {
        true => voxel_size / voxel_type_size,
        false => return Err("Invalid voxel size")
    };

    let class = voxel_type_to_mx_class_id(voxel_type);
    let shape_arr = [num_channels, shape.x as usize, shape.y as usize, shape.z as usize];

    let shape = match num_channels {
        1 => &shape_arr[1..],
        _ => &shape_arr[0..]
    };

    // read data
    let arr = create_numeric_array(shape, class, MxComplexity::Real)?;
    let mut mat = mx_array_mut_to_wkwrap_mat(arr)?;
    dataset.read_mat(bbox.min(), &mut mat)?;

    // set output
    lhs[0] = arr;

    Ok(())
});
