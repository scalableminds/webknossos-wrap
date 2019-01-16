extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;
use std::slice;
use std::path::Path;

unsafe fn new(nrhs: c_int, rhs: *const MxArray) -> Result<()> {
    let rhs = match nrhs == 5 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments")
    };

    let wkw_path = Path::new(mx_array_to_str(rhs[0])?);
    let block_len_log2 = as_log2(mx_array_to_f64(rhs[1])?)?;
    let file_len_log2 = as_log2(mx_array_to_f64(rhs[2])?)?;

    let class_id_name = mx_array_to_str(rhs[3])?;
    let class_id = str_slice_to_mx_class_id(class_id_name)?;
    let voxel_type = mx_class_id_to_voxel_type(class_id)?;

    let voxel_type_size = voxel_type.size();
    let block_type = wkwrap::BlockType::Raw;

    let num_elem = as_nat(mx_array_to_f64(rhs[4])?)?;
    let voxel_size = voxel_type_size as u8 * num_elem as u8;

    let header = wkwrap::Header {
        version: 2,
        block_len_log2: block_len_log2,
        file_len_log2: file_len_log2,
        block_type: block_type,
        voxel_type: voxel_type,
        voxel_size: voxel_size,
        data_offset: 0,
        jump_table: None
    };

    // create dataset
    wkwrap::Dataset::create(&wkw_path, header)?;

    Ok(())
}

unsafe fn compress(nrhs: c_int, rhs: *const MxArray) -> Result<()> {
    let rhs = match nrhs == 2 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments")
    };

    let src_path = Path::new(mx_array_to_str(rhs[0])?);
    let dst_path = Path::new(mx_array_to_str(rhs[1])?);

    let dataset = wkwrap::Dataset::new(&src_path)?;
    dataset.compress(&dst_path)?;

    Ok(())
}

mex_function!(_nlhs, _lhs, nrhs, rhs, {
    let command = match nrhs < 1 {
        true => Err("Not enough input arguments"),
        false => mx_array_to_str(*rhs)
    }?;

    match command {
        "new" => new(nrhs - 1, rhs.offset(1)),
        "compress" => compress(nrhs - 1, rhs.offset(1)),
        _ => Err("Unknown command")
    }
});
