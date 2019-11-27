use ::ffi::*;
use ::util::*;
use ::wkwrap;

fn f64_slice_to_wkwrap_vec(buf: &[f64]) -> Result<wkwrap::Vec3> {
    match buf.len() == 3 {
        true => Ok(wkwrap::Vec3 {
            x: as_nat(buf[0]).or(Err("Invalid X value"))? as u32,
            y: as_nat(buf[1]).or(Err("Invalid Y value"))? as u32,
            z: as_nat(buf[2]).or(Err("Invalid Z value"))? as u32
        }),
        false => Err("Size mismatch")
    }
}

pub fn mx_array_to_wkwrap_box(pm: MxArray) -> Result<wkwrap::Box3> {
    let buf = mx_array_to_f64_slice(pm)?;

    // verify shape of array
    if mx_array_size_to_usize_slice(pm) != &[3, 2] {
        return Err("Bounding box has invalid shape");
    }

    wkwrap::Box3::new(
        f64_slice_to_wkwrap_vec(&buf[0..3]).or(Err("Invalid lower bound"))? - 1,
        f64_slice_to_wkwrap_vec(&buf[3..6]).or(Err("Invalid upper bound"))?
    )
}

pub fn mx_array_to_wkwrap_vec(pm: MxArray) -> Result<wkwrap::Vec3> {
    let buf = mx_array_to_f64_slice(pm)?;
    f64_slice_to_wkwrap_vec(buf)
}

pub fn mx_array_to_wkwrap_mat<'a>(is_multi_channel: bool, pm: MxArray) -> Result<wkwrap::Mat<'a>> {
    // HACK(amotta): Ideally, we would also have wkwrap::MatMut
    mx_array_mut_to_wkwrap_mat(is_multi_channel, pm as MxArrayMut)
}

pub fn mx_class_id_to_voxel_type(class_id: MxClassId) -> Result<wkwrap::VoxelType> {
    match class_id {
        MxClassId::Uint8  => Ok(wkwrap::VoxelType::U8),
        MxClassId::Uint16 => Ok(wkwrap::VoxelType::U16),
        MxClassId::Uint32 => Ok(wkwrap::VoxelType::U32),
        MxClassId::Uint64 => Ok(wkwrap::VoxelType::U64),
        MxClassId::Single => Ok(wkwrap::VoxelType::F32),
        MxClassId::Double => Ok(wkwrap::VoxelType::F64),
        MxClassId::Int8   => Ok(wkwrap::VoxelType::I8),
        MxClassId::Int16  => Ok(wkwrap::VoxelType::I16),
        MxClassId::Int32  => Ok(wkwrap::VoxelType::I32),
        MxClassId::Int64  => Ok(wkwrap::VoxelType::I64),
        _                 => Err("Unknown MxClassId")
    }
}

pub fn voxel_type_to_mx_class_id(voxel_type: wkwrap::VoxelType) -> MxClassId {
    match voxel_type {
        wkwrap::VoxelType::U8  => MxClassId::Uint8,
        wkwrap::VoxelType::U16 => MxClassId::Uint16,
        wkwrap::VoxelType::U32 => MxClassId::Uint32,
        wkwrap::VoxelType::U64 => MxClassId::Uint64,
        wkwrap::VoxelType::F32 => MxClassId::Single,
        wkwrap::VoxelType::F64 => MxClassId::Double,
        wkwrap::VoxelType::I8  => MxClassId::Int8,
        wkwrap::VoxelType::I16 => MxClassId::Int16,
        wkwrap::VoxelType::I32 => MxClassId::Int32,
        wkwrap::VoxelType::I64 => MxClassId::Int64,
    }
}

pub fn mx_array_mut_to_wkwrap_mat<'a>(
    is_multi_channel: bool,
    pm: MxArrayMut
) -> Result<wkwrap::Mat<'a>> {
    // buffer
    let buf = mx_array_mut_to_u8_slice_mut(pm)?;

    // full size vector
    let mx_size = mx_array_size_to_usize_slice(pm);
    let mx_size_len = mx_size.len();

    // check number of input dimensions
    if mx_size_len > if is_multi_channel { 4 } else { 3 } {
        return Err("Data array has too many dimensions");
    }

    let mut size = [1usize; 4];
    let size_off = if is_multi_channel { 0 } else { 1 };
    size[size_off..(size_off + mx_size_len)].copy_from_slice(mx_size);

    // shape
    let shape = wkwrap::Vec3 { x: size[1] as u32, y: size[2] as u32, z: size[3] as u32 };

    // voxel size
    let elem_size = unsafe { mxGetElementSize(pm) };
    let voxel_size = elem_size * size[0];

    // voxel type
    let voxel_type = mx_class_id_to_voxel_type(unsafe { mxGetClassID(pm) })?;

    wkwrap::Mat::new(buf, shape, voxel_size, voxel_type, false)
}
