extern crate wkw;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::option::Option;

use std::io;

fn main() {
    let wkw_path = "/home/amotta/Desktop/test.wkw";

    let mut file = File::open(wkw_path).unwrap();
    let mut wkw_file = wkw::File::new(&mut file).unwrap();

    println!("Header: {:#?}", wkw_file.header());

    // allocate buffer matrix
    let mut buf = vec![0 as u8; 128 * 128 * 128];

    let mut buf_mat = wkw::Mat::new(
        buf.as_mut_slice(),
        wkw::Vec::from_scalar(128 as u32),
        wkw_file.header().voxel_size as usize).unwrap();
    let pos = wkw::Vec::from_scalar(128);

    wkw_file.read_mat(&mut buf_mat, &pos).unwrap();

    println!("{:#?}", buf_mat);
}
