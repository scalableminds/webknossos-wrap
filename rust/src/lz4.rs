extern crate lz4 as lz4_crate;
use self::lz4_crate::liblz4;
use Result;

pub fn compress_bound(input_size: usize) -> usize {
    unsafe { liblz4::LZ4_compressBound(input_size as i32) as usize }
}

pub fn compress_hc(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
    let src_size = src_buf.len() as i32;
    let dst_capacity = dst_buf.len() as i32;
    let compression_level = 9;

    let dst_len = unsafe {
        liblz4::LZ4_compress_HC(
            std::mem::transmute::<&[u8], &[i8]>(src_buf).as_ptr(),
            std::mem::transmute::<&mut [u8], &mut [i8]>(dst_buf).as_mut_ptr(),
            src_size,
            dst_capacity,
            compression_level,
        )
    };

    match dst_len == 0 {
        true => Err(String::from("Error in LZ4_compress_HC")),
        false => Ok(dst_len as usize),
    }
}

pub fn decompress_safe(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
    let compressed_size = src_buf.len() as i32;
    let max_decompressed_size = dst_buf.len() as i32;

    let dst_len = unsafe {
        liblz4::LZ4_decompress_safe(
            std::mem::transmute::<&[u8], &[i8]>(src_buf).as_ptr(),
            std::mem::transmute::<&mut [u8], &mut [i8]>(dst_buf).as_mut_ptr(),
            compressed_size,
            max_decompressed_size,
        )
    };

    match dst_len < 0 {
        true => Err(String::from("Error in LZ4_decompress_safe")),
        false => Ok(dst_len as usize),
    }
}
