use ::ffi::*;
use ::util::*;
use ::wkwrap;

use std::cmp;

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

pub fn voxel_type_to_mx_class_id(voxel_type: wkwrap::VoxelType) -> MxClassId {
    match voxel_type {
        wkwrap::VoxelType::U8  => MxClassId::Uint8,
        wkwrap::VoxelType::U16 => MxClassId::Uint16,
        wkwrap::VoxelType::U32 => MxClassId::Uint32,
        wkwrap::VoxelType::U64 => MxClassId::Uint64,
        wkwrap::VoxelType::F32 => MxClassId::Single,
        wkwrap::VoxelType::F64 => MxClassId::Double
    }
}

pub fn mx_array_mut_to_wkwrap_mat<'a>(pm: MxArrayMut) -> Result<wkwrap::Mat<'a>> {
    let buf = mx_array_mut_to_u8_slice_mut(pm)?;
    let elem_size = unsafe { mxGetElementSize(pm) };
    let voxel_type = mx_class_id_to_voxel_type(unsafe { mxGetClassID(pm) })?;

    let size_mx = mx_array_size_to_usize_slice(pm);
    let ndim_mx = size_mx.len();

    if ndim_mx < 1 || ndim_mx > 4 {
        return Err("Matrix must be one-, two- or three-dimensional");
    }

    // MATLAB silently drops trailing singleton dimensions. If, for example, a
    // three-dimensional matrix with size [512, 1, 1] is created, then
    // mxGetNumberOfDimensions (and thus also mx_array_size_to_usize_slice)
    // only report a single dimension.
    let mut size = [1usize; 4];
    size[..ndim_mx].copy_from_slice(size_mx);
    let ndim = cmp::max(3, ndim_mx);

    let voxel_size = if ndim < 4 {
        elem_size
    } else {
        elem_size * size[0]
    };

    let shape = wkwrap::Vec3 {
        x: size[ndim - 3] as u32,
        y: size[ndim - 2] as u32,
        z: size[ndim - 1] as u32
    };

    match elem_size == 0 {
        true => Err("Failed to determine element size"),
        false => wkwrap::Mat::new(buf, shape, voxel_size, voxel_type)
    }
}
