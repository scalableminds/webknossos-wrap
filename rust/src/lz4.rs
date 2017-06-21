extern crate libc;
use self::libc::c_int;
use ::Result;

#[link(name = "lz4")]
extern {
    // decompression
    fn LZ4_decompress_safe(
        src_buf: *const u8,
        dst_buf: *mut u8,
        compressed_size: c_int,
        max_decompressed_size: c_int) -> c_int;
}

pub fn decompress_safe(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
    let compressed_size = src_buf.len() as c_int;
    let max_decompressed_size = dst_buf.len() as c_int;

    let dst_len = unsafe {
        LZ4_decompress_safe(
            src_buf.as_ptr(),
            dst_buf.as_mut_ptr(),
            compressed_size,
            max_decompressed_size)
    };

    match dst_len < 0 {
        true => Err("Error in LZ4_decompress_safe"),
        false => Ok(dst_len as usize)
    }
}
