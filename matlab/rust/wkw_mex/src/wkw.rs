use ::ffi::*;
use ::util::*;
use ::wkwrap;

use std::mem;

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

#[allow(mutable_transmutes)]
pub fn mx_array_to_wkwrap_mat<'a>(pm: MxArray) -> Result<wkwrap::Mat<'a>> {
    let buf = mx_array_to_u8_slice(pm)?;
    let shape = mx_array_size_to_wkwrap_vec(pm)?;
    let elem_size = unsafe { mxGetElementSize(pm) };

    // TODO(amotta): This is extremely dangerous
    let buf: &mut [u8] = unsafe { mem::transmute(buf) };

    match elem_size == 0 {
        true => Err("Failed to determine element size"),
        false => wkwrap::Mat::new(buf, shape, elem_size)
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
