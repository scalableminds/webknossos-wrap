extern crate libc;
use self::libc::c_int;
use ::Result;

#[link(name = "lz4")]
extern {
    // upper bound
    fn LZ4_compressBound(input_size: c_int) -> c_int;

    // compression
    fn LZ4_compress_HC(
        src: *const u8,
        dst: *mut u8,
        src_size: c_int,
        dst_capacity: c_int,
        compression_level: c_int
    ) -> c_int;

    // decompression
    fn LZ4_decompress_safe(
        src_buf: *const u8,
        dst_buf: *mut u8,
        compressed_size: c_int,
        max_decompressed_size: c_int
    ) -> c_int;
}

pub fn compress_bound(input_size: usize) -> usize {
    unsafe { LZ4_compressBound(input_size as c_int) as usize }
}

pub fn compress_hc(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
    let src_size = src_buf.len() as c_int;
    let dst_capacity = dst_buf.len() as c_int;
    let compression_level = 9;

    let dst_len = unsafe {
        LZ4_compress_HC(
            src_buf.as_ptr(),
            dst_buf.as_mut_ptr(),
            src_size,
            dst_capacity,
            compression_level
        )
    };

    match dst_len == 0 {
        true => Err("Error in LZ4_compress_HC"),
        false => Ok(dst_len as usize)
    }
}

pub fn decompress_safe(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
    let compressed_size = src_buf.len() as c_int;
    let max_decompressed_size = dst_buf.len() as c_int;

    let dst_len = unsafe {
        LZ4_decompress_safe(
            src_buf.as_ptr(),
            dst_buf.as_mut_ptr(),
            compressed_size,
            max_decompressed_size
        )
    };

    match dst_len < 0 {
        true => Err("Error in LZ4_decompress_safe"),
        false => Ok(dst_len as usize)
    }
}
