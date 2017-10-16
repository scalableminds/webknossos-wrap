extern crate wkwrap;
use wkwrap as wkw;

use std::ffi::CStr;
use std::os::raw::{c_char, c_void, c_ulong};

use std::path::Path;

pub enum Dataset {}

#[repr(C)]
pub struct Header {
    pub version: u8,
    pub block_len: u8,
    pub file_len: u8,
    pub block_type: u8,
    pub voxel_type: u8,
    pub voxel_size: u8
}

#[no_mangle]
pub extern fn dataset_open(root_ptr: *const c_char) -> *const Dataset {
    let root_str = unsafe { CStr::from_ptr(root_ptr) }.to_str().unwrap();
    let root_path = Path::new(root_str);

    let dataset = wkw::Dataset::new(root_path).unwrap();
    let dataset_ptr = Box::from(dataset);

    unsafe {
        std::mem::transmute(dataset_ptr)
    }
}

#[no_mangle]
pub extern fn dataset_close(dataset_ptr: *const Dataset) {
    assert!(!dataset_ptr.is_null());

    #[allow(unused_variables)]
    let dataset = unsafe { Box::from_raw(dataset_ptr as *mut wkwrap::Dataset) };

    // NOTE(amotta): At this point the liftime or the `dataset` binding will end
    // and the Rust language will make sure that the Dataset structure is cleared.
}

#[no_mangle]
pub extern fn dataset_get_header(dataset_ptr: *const Dataset, header_ptr: *mut Header) {
    assert!(!dataset_ptr.is_null());
    assert!(!header_ptr.is_null());

    let dataset = unsafe { Box::from_raw(dataset_ptr as *mut wkwrap::Dataset) };

    unsafe {
        let header = dataset.header();
        (*header_ptr).version = header.version;
        (*header_ptr).block_len = 1u8 << header.block_len_log2;
        (*header_ptr).file_len = 1u8 << header.file_len_log2;
        (*header_ptr).block_type = 1u8 + header.block_type as u8;
        (*header_ptr).voxel_type = 1u8 + header.voxel_type as u8;
        (*header_ptr).voxel_size = header.voxel_size;
    }

    std::mem::forget(dataset);
}

#[no_mangle]
pub extern fn dataset_read(
    dataset_ptr: *const Dataset,
    bbox_ptr: *const c_ulong,
    data_ptr: *mut c_void
) {
    assert!(!dataset_ptr.is_null());
    assert!(!bbox_ptr.is_null());
    assert!(!data_ptr.is_null());

    let dataset = unsafe { Box::from_raw(dataset_ptr as *mut wkwrap::Dataset) };
    let bbox = unsafe { std::slice::from_raw_parts(bbox_ptr as *const u32, 6) };

    let off = wkwrap::Vec3 { x: bbox[0], y: bbox[1], z: bbox[2] };
    let shape = wkwrap::Vec3 { x: bbox[3] - bbox[0], y: bbox[4] - bbox[1], z: bbox[5] - bbox[2] };

    let voxel_type = dataset.header().voxel_type;
    let voxel_size = dataset.header().voxel_size as usize;

    let data_len = shape.product() as usize * voxel_size;
    let data = unsafe { std::slice::from_raw_parts_mut(data_ptr as *mut u8, data_len) };

    let mut mat = wkwrap::Mat::new(data, shape, voxel_size, voxel_type).unwrap();
    dataset.read_mat(off, &mut mat).unwrap();

    std::mem::forget(dataset);
}

