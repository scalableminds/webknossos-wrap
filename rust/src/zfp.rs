extern crate libc;
use self::libc::c_int;
use self::libc::size_t;
// use self::libc::c_uint;
use Result;

#[cfg_attr(target_os = "linux", link(name = "zfp", kind = "static"))]     // libzfp does not work as name
#[cfg_attr(target_os = "macos", link(name = "zfp", kind = "static"))]
#[cfg_attr(target_os = "windows", link(name = "libzfp", kind = "static"))]
extern "C" {
    // upper bound
    // fn zfp_compressBound(input_size: c_int) -> c_int;
    fn zfp_stream_open() -> c_int;
    // fn estimate_maximum_byte_size(unsigned int cube_length, unsigned int precision);
    fn zfp_wrapped_estimate_maximum_byte_size(cube_length: usize, precision: usize) -> usize;

    fn zfp_wrapped_compress(
    	src: *const u8,
    	dst: *mut u8,
    	output_buffer_byte_size: usize,
    	cube_length: usize,
    	precision: usize,
    ) -> usize;

    fn zfp_wrapped_decompress(
    	compressed_array: *const u8,
    	decompressed_array: *mut u8,
    	num_compressed_elements: size_t,
    	cube_length: size_t,
    	precision: usize,
    );

    // size_t zfp_wrapped_compress(
    // 	const uint8 * input_array,
    // 	uint8 * output_array,
    // 	size_t output_buffer_byte_size,
    // 	size_t cube_length,
    // 	unsigned int precision
    // )

    // // compression
    // fn zfp_compress_HC(
    //     src: *const u8,
    //     dst: *mut u8,
    //     src_size: c_int,
    //     dst_capacity: c_int,
    //     compression_level: c_int,
    // ) -> c_int;

    // // decompression
    // fn zfp_decompress_safe(
    //     src_buf: *const u8,
    //     dst_buf: *mut u8,
    //     compressed_size: c_int,
    //     max_decompressed_size: c_int,
    // ) -> c_int;
}

pub fn compress_bound(input_size: usize) -> usize {
	// 0
    unsafe {
    	// zfp_compressBound(input_size as c_int) as usize
    	zfp_stream_open() as usize
    }
}

pub fn estimate_maximum_byte_size(cube_length: usize, precision: usize) -> usize {
	unsafe {
		zfp_wrapped_estimate_maximum_byte_size(cube_length as size_t, precision as size_t) as usize
	}
}


pub fn zfp_compress(
	src: &[u8],
	dst: &mut [u8],
	output_buffer_byte_size: usize,
	cube_length: usize,
	precision: usize,
) -> Result<usize> {
	unsafe {
		let bits = zfp_wrapped_compress(src.as_ptr(), dst.as_mut_ptr(), output_buffer_byte_size as size_t, cube_length as size_t, precision as size_t) as usize;

		match bits == 0 {
			true => Err("Error in zfp_compress"),
			false => Ok(bits as usize),
		}
	}
}

pub fn zfp_decompress(
	compressed_array: &[u8],
	decompressed_array: &mut [u8],
	num_compressed_elements: usize,
	cube_length: usize,
	precision: usize,
) -> Result<usize> {
	unsafe {
		zfp_wrapped_decompress(
			compressed_array.as_ptr(),
			decompressed_array.as_mut_ptr(),
			num_compressed_elements as size_t,
			cube_length as size_t,
			precision as size_t,
		);
		Ok(1 as usize)
	}
}


pub fn compress_hc(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
	Err("Not implemented")
    // let src_size = src_buf.len() as c_int;
    // let dst_capacity = dst_buf.len() as c_int;
    // let compression_level = 9;

    // let dst_len = unsafe {
    //     zfp_compress_HC(
    //         src_buf.as_ptr(),
    //         dst_buf.as_mut_ptr(),
    //         src_size,
    //         dst_capacity,
    //         compression_level,
    //     )
    // };

    // match dst_len == 0 {
    //     true => Err("Error in zfp_compress_HC"),
    //     false => Ok(dst_len as usize),
    // }
}

pub fn decompress_safe(src_buf: &[u8], dst_buf: &mut [u8]) -> Result<usize> {
	Err("Not implemented")
    // let compressed_size = src_buf.len() as c_int;
    // let max_decompressed_size = dst_buf.len() as c_int;

    // let dst_len = unsafe {
    //     zfp_decompress_safe(
    //         src_buf.as_ptr(),
    //         dst_buf.as_mut_ptr(),
    //         compressed_size,
    //         max_decompressed_size,
    //     )
    // };

    // match dst_len < 0 {
    //     true => Err("Error in zfp_decompress_safe"),
    //     false => Ok(dst_len as usize),
    // }
}
