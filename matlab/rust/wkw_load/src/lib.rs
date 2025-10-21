extern crate wkwrap;
extern crate zarrs;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;
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

    let array_shape = array.shape();
    let ndim = array_shape.len();
    let data_type = array.data_type();
    let type_size = if let Some(type_size) = data_type.fixed_size() {
        type_size
    } else {
        return Err("Unsupported data type".to_string());
    };

    // build shape
    let (bbox_start, bbox_shape) = mx_array_to_bbox(rhs[1], ndim)?;

    if bbox_start
        .iter()
        .zip(bbox_shape.iter())
        .zip(array_shape.iter())
        .any(|((bbox_min_x, bbox_shape_x), shape_x)| (*bbox_min_x + *bbox_shape_x) > *shape_x)
    {
        return Err(format!(
            "Bounding box start={:?}, shape={:?} is out of bounds for array of shape={:?}.",
            bbox_start, bbox_shape, array_shape
        ));
    }
    let subset = zarrs_result_to_str_error(ArraySubset::new_with_start_shape(
        bbox_start.clone(),
        bbox_shape.clone(),
    ))?;

    // prepare allocation
    let mat_class = match array.data_type() {
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
    let zarr_buf = zarrs_result_to_str_error(data_all.into_fixed())?.into_owned(); // in c-order

    let mat_arr = create_numeric_array(&bbox_shape, mat_class, MxComplexity::Real)?;

    copy_as_fortran_order(&zarr_buf, mat_arr, &bbox_shape, type_size)?;

    // set output
    lhs[0] = mat_arr;

    Ok(())
});
