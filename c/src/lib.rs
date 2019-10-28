extern crate wkwrap;
use wkwrap as wkw;

#[macro_use]
extern crate lazy_static;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_ulong, c_void};

use std::path::Path;
use std::sync::Mutex;

pub enum Dataset {}

#[repr(C)]
pub struct Header {
    pub version: u8,
    pub block_len: u8,
    pub file_len: u8,
    pub block_type: u8,
    pub voxel_type: u8,
    pub voxel_size: u8,
}

fn as_log2(i: u8) -> Result<u8, &'static str> {
    match i & (i - 1) == 0 {
        true => Ok(i.trailing_zeros() as u8),
        false => Err("Input must be a power of two"),
    }
}

fn from_header(header_ptr: *const Header) -> Result<wkw::Header, &'static str> {
    assert!(!header_ptr.is_null());

    let c_header = unsafe { &*header_ptr };

    let block_type = match c_header.block_type {
        1 => wkw::BlockType::Raw,
        2 => wkw::BlockType::LZ4,
        3 => wkw::BlockType::LZ4HC,
        _ => return Err("Block type is invalid"),
    };

    let voxel_type = match c_header.voxel_type {
        1 => wkw::VoxelType::U8,
        2 => wkw::VoxelType::U16,
        3 => wkw::VoxelType::U32,
        4 => wkw::VoxelType::U64,
        5 => wkw::VoxelType::F32,
        6 => wkw::VoxelType::F64,
        7 => wkw::VoxelType::I8,
        8 => wkw::VoxelType::I16,
        9 => wkw::VoxelType::I32,
        10 => wkw::VoxelType::I64,
        _ => return Err("Voxel type is invalid"),
    };

    let block_len_log2 = as_log2(c_header.block_len)?;
    let file_len_log2 = as_log2(c_header.file_len)?;

    Ok(wkw::Header {
        version: c_header.version,
        block_len_log2: block_len_log2,
        file_len_log2: file_len_log2,
        block_type: block_type,
        voxel_type: voxel_type,
        voxel_size: c_header.voxel_size,
        data_offset: 0,
        jump_table: None,
    })
}

fn check_return<T>(ret: Result<T, &str>) -> c_int {
    match ret {
        Ok(_) => 0,
        Err(msg) => {
            set_last_error_msg(msg);
            1
        }
    }
}

lazy_static! {
    static ref LAST_ERR_MSG: Mutex<Box<CStr>> =
        Mutex::new(CString::new("".as_bytes()).unwrap().into_boxed_c_str());
}

#[no_mangle]
pub extern "C" fn get_last_error_msg() -> *const c_char {
    LAST_ERR_MSG.lock().unwrap().as_ptr()
}

fn set_last_error_msg(msg: &str) {
    let c_string = CString::new(msg.as_bytes()).unwrap();
    *LAST_ERR_MSG.lock().unwrap() = c_string.into_boxed_c_str();
}

#[no_mangle]
pub extern "C" fn dataset_open(root_ptr: *const c_char) -> *const Dataset {
    assert!(!root_ptr.is_null());
    let root_str = unsafe { CStr::from_ptr(root_ptr) }.to_str().unwrap();
    let root_path = Path::new(root_str);

    match wkw::Dataset::new(root_path) {
        Ok(dataset) => {
            let dataset_ptr = Box::from(dataset);
            unsafe { std::mem::transmute(dataset_ptr) }
        }
        Err(msg) => {
            set_last_error_msg(msg);
            std::ptr::null::<Dataset>()
        }
    }
}

#[no_mangle]
pub extern "C" fn dataset_close(dataset_ptr: *const Dataset) {
    assert!(!dataset_ptr.is_null());

    #[allow(unused_variables)]
    let dataset = unsafe { Box::from_raw(dataset_ptr as *mut wkwrap::Dataset) };

    // NOTE(amotta): At this point the liftime or the `dataset` binding will end
    // and the Rust language will make sure that the Dataset structure is cleared.
}

#[no_mangle]
pub extern "C" fn dataset_get_header(dataset_ptr: *const Dataset, header_ptr: *mut Header) {
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
pub extern "C" fn dataset_create(
    root_ptr: *const c_char,
    header_ptr: *const Header,
) -> *const Dataset {
    assert!(!header_ptr.is_null());
    assert!(!root_ptr.is_null());

    let root_str = unsafe { CStr::from_ptr(root_ptr) }.to_str().unwrap();
    let root_path = Path::new(root_str);

    match from_header(header_ptr).and_then(|header| wkw::Dataset::create(root_path, header)) {
        Ok(dataset) => {
            let dataset_ptr = Box::from(dataset);
            unsafe { std::mem::transmute(dataset_ptr) }
        }
        Err(msg) => {
            set_last_error_msg(msg);
            std::ptr::null::<Dataset>()
        }
    }
}

#[no_mangle]
pub extern "C" fn file_compress(src_path_ptr: *const c_char, dst_path_ptr: *const c_char) -> c_int {
    assert!(!src_path_ptr.is_null());
    assert!(!dst_path_ptr.is_null());

    let src_path_str = unsafe { CStr::from_ptr(src_path_ptr) }.to_str().unwrap();
    let src_path = Path::new(src_path_str);
    let dst_path_str = unsafe { CStr::from_ptr(dst_path_ptr) }.to_str().unwrap();
    let dst_path = Path::new(dst_path_str);

    check_return(wkwrap::File::open(&src_path).and_then(|mut file| file.compress(&dst_path)))
}

fn c_bbox_to_off_and_shape(bbox_ptr: *const c_ulong) -> (wkwrap::Vec3, wkwrap::Vec3) {
    let bbox = unsafe { std::slice::from_raw_parts(bbox_ptr as *const u32, 6) };

    let off = wkwrap::Vec3 {
        x: bbox[0],
        y: bbox[1],
        z: bbox[2],
    };

    let shape = wkwrap::Vec3 {
        x: bbox[3] - bbox[0],
        y: bbox[4] - bbox[1],
        z: bbox[5] - bbox[2],
    };

    (off, shape)
}

fn c_data_to_mat<'a>(
    dataset: &wkwrap::Dataset,
    shape: &'a wkwrap::Vec3,
    data_ptr: *const c_void,
    data_in_c_order: bool,
) -> wkwrap::Mat<'a> {
    let voxel_type = dataset.header().voxel_type;
    let voxel_size = dataset.header().voxel_size as usize;

    let data_len = shape.product() as usize * voxel_size;
    let data = unsafe { std::slice::from_raw_parts_mut(data_ptr as *mut u8, data_len) };

    wkwrap::Mat::new(data, *shape, voxel_size, voxel_type, data_in_c_order).unwrap()
}

#[no_mangle]
pub extern "C" fn dataset_read(
    dataset_ptr: *const Dataset,
    bbox_ptr: *const c_ulong,
    data_ptr: *mut c_void,
) -> c_int {
    assert!(!dataset_ptr.is_null());
    assert!(!bbox_ptr.is_null());
    assert!(!data_ptr.is_null());

    let dataset = unsafe { Box::from_raw(dataset_ptr as *mut wkwrap::Dataset) };
    let (off, shape) = c_bbox_to_off_and_shape(bbox_ptr);

    let mut mat = c_data_to_mat(&dataset, &shape, data_ptr, false);
    let ret = dataset.read_mat(off, &mut mat);
    std::mem::forget(dataset);
    check_return(ret)
}

#[no_mangle]
pub extern "C" fn dataset_write(
    dataset_ptr: *const Dataset,
    bbox_ptr: *const c_ulong,
    data_ptr: *const c_void,
    data_in_c_order: bool,
) -> c_int {
    assert!(!dataset_ptr.is_null());
    assert!(!bbox_ptr.is_null());
    assert!(!data_ptr.is_null());

    let dataset = unsafe { Box::from_raw(dataset_ptr as *mut wkwrap::Dataset) };

    let (off, shape) = c_bbox_to_off_and_shape(bbox_ptr);

    let mat = c_data_to_mat(&dataset, &shape, data_ptr, data_in_c_order);
    let ret = dataset.write_mat(off, &mat);
    std::mem::forget(dataset);
    check_return(ret)
}
