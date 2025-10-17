extern crate wkwrap;
extern crate zarrs;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;
use wkwrap::{Box3, Mat, Vec3, VoxelType};
use zarrs::array::data_type::DataType;
use zarrs::array::Array;
use zarrs::array_subset::ArraySubset;

use std::path::PathBuf;
use std::slice;
use std::sync::Arc;

fn zarrs_result_to_str_error<T, E: std::error::Error>(
    result: std::result::Result<T, E>,
) -> Result<T> {
    match result {
        Ok(ok) => Ok(ok),
        Err(err) => Err(err.to_string()),
    }
}

mex_function!(nlhs, lhs, nrhs, rhs, {
    let rhs = match nrhs == 2 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments".to_string()),
    };

    let lhs = match nlhs == 1 {
        true => slice::from_raw_parts_mut(lhs, nlhs as usize),
        false => return Err("Invalid number of output arguments".to_string()),
    };

    let store_path: PathBuf = mx_array_to_str(rhs[0])?.into();

    let store: zarrs::storage::ReadableWritableListableStorage = Arc::new(
        zarrs_result_to_str_error(zarrs::filesystem::FilesystemStore::new(&store_path))?,
    );
    let array = zarrs_result_to_str_error(Array::open(store.clone(), "/"))?;

    let num_channels = array.shape()[0] as usize;
    let is_multi_channel = num_channels > 1;

    // build shape
    let bbox = mx_array_to_wkwrap_box(rhs[1])?;
    let subset = zarrs_result_to_str_error(ArraySubset::new_with_start_shape(
        vec![
            0,
            bbox.min().x as u64,
            bbox.min().y as u64,
            bbox.min().z as u64,
        ],
        vec![
            1,
            bbox.width().x as u64,
            bbox.width().y as u64,
            bbox.width().z as u64,
        ],
    ))?;

    let shape_arr = [
        num_channels,
        bbox.width().x as usize,
        bbox.width().y as usize,
        bbox.width().z as usize,
    ];
    let shape_slice = if is_multi_channel {
        &shape_arr[0..]
    } else {
        &shape_arr[1..]
    };

    // prepare allocation
    let voxel_type = match array.data_type() {
        DataType::UInt8 => VoxelType::U8,
        DataType::UInt16 => VoxelType::U16,
        DataType::UInt32 => VoxelType::U32,
        DataType::UInt64 => VoxelType::U64,
        DataType::Float32 => VoxelType::F32,
        DataType::Float64 => VoxelType::F64,
        DataType::Int8 => VoxelType::I8,
        DataType::Int16 => VoxelType::I16,
        DataType::Int32 => VoxelType::I32,
        DataType::Int64 => VoxelType::I64,
        _ => {
            return Err("Unsupported data type".to_string());
        }
    };
    let class = match array.data_type() {
        DataType::UInt8 => MxClassId::Uint8,
        DataType::UInt16 => MxClassId::Uint16,
        DataType::UInt32 => MxClassId::Uint32,
        DataType::UInt64 => MxClassId::Uint64,
        DataType::Float32 => MxClassId::Single,
        DataType::Float64 => MxClassId::Double,
        DataType::Int8 => MxClassId::Int8,
        DataType::Int16 => MxClassId::Int16,
        DataType::Int32 => MxClassId::Int32,
        DataType::Int64 => MxClassId::Int64,
        _ => {
            return Err("Unsupported data type".to_string());
        }
    };

    // read data
    let data_all = zarrs_result_to_str_error(array.retrieve_array_subset(&subset))?;
    let mut buf = zarrs_result_to_str_error(data_all.into_fixed())?.into_owned(); // in c-order
    let src_mat = Mat::new(
        &mut buf,
        bbox.width(),
        voxel_type.size() * num_channels,
        voxel_type,
        true,
    )?;
    let arr = create_numeric_array(shape_slice, class, MxComplexity::Real)?;
    let mut mat = mx_array_mut_to_wkwrap_mat(is_multi_channel, arr)?;

    src_mat.copy_as_fortran_order(
        &mut mat,
        Box3::new(Vec3 { x: 0, y: 0, z: 0 }, bbox.width())?,
    )?;

    // set output
    lhs[0] = arr;

    Ok(())
});
