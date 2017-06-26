extern crate wkwrap;

#[macro_use]
extern crate wkw_mex;
use wkw_mex::*;
use std::slice;
use std::path::Path;

fn try_nat(f: f64) -> Result<u64> {
    if f <= 0.0 {
        return Err("Input must be positive");
    }

    match f % 1.0 == 0.0 {
        true => Ok(f as u64),
        false => Err("Input must be an integer")
    }
}

fn try_log2(f: f64) -> Result<u8> {
    let i = try_nat(f)?;

    match i & (i - 1) == 0 {
        true => Ok(i.trailing_zeros() as u8),
        false => Err("Input must be a power of two")
    }
}

#[no_mangle]
mex_function!(_nlhs, _lhs, nrhs, rhs, {
    let rhs = match nrhs == 5 {
        true => slice::from_raw_parts(rhs, nrhs as usize),
        false => return Err("Invalid number of input arguments")
    };

    let wkw_path = Path::new(mx_array_to_str(rhs[0])?);
    let block_len_log2 = try_log2(mx_array_to_f64(rhs[1])?)?;
    let file_len_log2 = try_log2(mx_array_to_f64(rhs[2])?)?;

    let voxel_type = match mx_array_to_str(rhs[3])? {
        "uint8" => wkwrap::VoxelType::U8,
        "uint32" => wkwrap::VoxelType::U32,
        _ => return Err("Unsupported voxel type")
    };

    let voxel_type_size = wkwrap::header::voxel_type_size(voxel_type);
    let block_type = wkwrap::BlockType::Raw;

    let num_elem = try_nat(mx_array_to_f64(rhs[4])?)?;
    let voxel_size = voxel_type_size as u8 * num_elem as u8;

    let header = wkwrap::Header {
        version: 1,
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
});
