use ::ffi::*;
use ::util::*;
use ::wkwrap;

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

pub fn mx_array_to_wkwrap_mat<'a>(pm: MxArray) -> Result<wkwrap::Mat<'a>> {
    // HACK(amotta): Ideally, we would also have wkwrap::MatMut
    mx_array_mut_to_wkwrap_mat(pm as MxArrayMut)
}

pub fn mx_class_id_to_voxel_type(class_id: MxClassId) -> Result<wkwrap::VoxelType> {
    match class_id {
        MxClassId::Uint8  => Ok(wkwrap::VoxelType::U8),
        MxClassId::Uint16 => Ok(wkwrap::VoxelType::U16),
        MxClassId::Uint32 => Ok(wkwrap::VoxelType::U32),
        MxClassId::Uint64 => Ok(wkwrap::VoxelType::U64),
        MxClassId::Single => Ok(wkwrap::VoxelType::F32),
        MxClassId::Double => Ok(wkwrap::VoxelType::F64),
        _                 => Err("Unknown MxClassId")
    }
}

pub fn mx_array_mut_to_wkwrap_mat<'a>(pm: MxArrayMut) -> Result<wkwrap::Mat<'a>> {
    let buf = mx_array_mut_to_u8_slice_mut(pm)?;
    let shape = mx_array_size_to_wkwrap_vec(pm)?;

    let elem_size = unsafe { mxGetElementSize(pm) };
    let voxel_type = mx_class_id_to_voxel_type(unsafe { mxGetClassID(pm) })?;

    match elem_size == 0 {
        true => Err("Failed to determine element size"),
        false => wkwrap::Mat::new(buf, shape, elem_size, voxel_type)
    }
}
