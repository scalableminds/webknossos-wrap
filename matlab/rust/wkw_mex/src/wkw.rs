use ::ffi::*;
use ::util::*;
use ::wkwrap;

use std::{mem, slice};

pub fn mx_array_to_wkwrap_vec(pm: MxArray) -> Result<wkwrap::Vec3> {
    let buf = mx_array_to_f64_slice(pm)?;

    match buf.len() == 3 {
        true => Ok(wkwrap::Vec3 {
            x: buf[0] as u32,
            y: buf[1] as u32,
            z: buf[2] as u32
        }),
        false => Err("Size mismatch")
    }
}

pub fn mx_array_size_to_wkwrap_vec(pm: MxArray) -> Result<wkwrap::Vec3> {
    let size = mx_array_size_to_usize_slice(pm);

    match size.len() == 3 {
        true => Ok(wkwrap::Vec3 {
            x: size[0] as u32,
            y: size[1] as u32,
            z: size[2] as u32
        }),
        false => Err("Dimensionality mismatch")
    }
}

pub fn mx_array_mut_to_wkwrap_mat<'a>(pm: MxArrayMut) -> Result<wkwrap::Mat<'a>> {
    let buf = mx_array_mut_to_u8_slice_mut(pm)?;
    let shape = mx_array_size_to_wkwrap_vec(pm)?;
    let elem_size = unsafe { mxGetElementSize(pm) };

    match elem_size == 0 {
        true => Err("Failed to determine element size"),
        false => wkwrap::Mat::new(buf, shape, elem_size)
    }
}

pub fn create_wkwrap_mat<'a>(
    shape: wkwrap::Vec3,
    voxel_size: usize,
    voxel_type: wkwrap::VoxelType)
-> Result<(MxArrayMut, wkwrap::Mat<'a>)> {

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

    // build buffer slice
    let arr_ptr = unsafe { mxGetData(arr) as *mut u8 };
    let numel = arr_shape.into_iter().fold(1, |a, &b| a * b);
    let buf = unsafe { slice::from_raw_parts_mut(arr_ptr, numel) };

    // build wkw matrix
    let mat = wkwrap::Mat::new(buf, shape, voxel_size)?;

    Ok((arr, mat))
}
