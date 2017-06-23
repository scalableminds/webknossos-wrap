extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;

use std::slice;
use std::path::Path;

#[no_mangle]
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
    let bbox = mx_array_to_f64_slice(rhs[1])?;

    let bbox = wkwrap::Box3::new(
        wkwrap::Vec3 { x: bbox[0] as u32, y: bbox[1] as u32, z: bbox[2] as u32 },
        wkwrap::Vec3 { x: bbox[3] as u32, y: bbox[4] as u32, z: bbox[5] as u32 }
    )?;

    let dataset_path = Path::new(wkw_path);
    let dataset = wkwrap::Dataset::new(dataset_path)?;

    // prepare allocation
    let shape = bbox.width();
    let voxel_size = dataset.header().voxel_size as usize;
    let voxel_type = dataset.header().voxel_type;

    let (type_size, class) = match voxel_type {
        wkwrap::VoxelType::U8 => (1 as usize, MxClassId::Uint8),
        wkwrap::VoxelType::U32 => (4 as usize, MxClassId::Uint32),
        _ => return Err("Unsupported voxel type")
    };

    let size_last = match voxel_size % type_size == 0 {
        true => voxel_size / type_size,
        false => return Err("Invalid voxel size")
    };

    // create MATLAB array
    let arr_shape = [shape.x as usize, shape.y as usize, shape.z as usize, size_last];
    let arr = create_uninit_numeric_array(&arr_shape, class, MxComplexity::Real)?;

    // read data
    let mut mat = mx_array_mut_to_wkwrap_mat(arr)?;
    dataset.read_mat(bbox.min(), &mut mat)?;

    // set output
    lhs[0] = arr;

    Ok(())
});
