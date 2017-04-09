use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;

use header::Header;
use result::Result;
use mat::Mat;
use vec::Vec;

#[derive(Debug)]
pub struct Dataset<'a> {
    root: &'a OsStr,
    header: Header
}

static HEADER_FILE_NAME: &'static str = "header.wkw";

impl<'a> Dataset<'a> {
    pub fn new(root: &'a Path) -> Result<Dataset<'a>> {
        if !root.is_dir() {
            return Err("Dataset root is not a directory")
        }

        // read required header file
        let header = Self::read_header(root)?;

        Ok(Dataset {
            root: root.as_os_str(),
            header: header
        })
    }

    pub fn header(&'a self) -> &'a Header { &self.header }

    pub fn read_mat(&mut self, mat: &mut Mat, off: &Vec) -> Result<usize> {
        let vec_min = off.clone() / self.header.block_len() as u32;
        let vec_max = (off.clone() + mat.shape() - 1u32) / self.header.block_len() as u32 + 1;

        for cur_x in vec_min.x..vec_max.x {
            for cur_y in vec_min.y..vec_max.y {
                for cur_z in vec_min.z..vec_max.z {
                    println!("X: {}, Y: {}, Z: {}", cur_x, cur_y, cur_z);
                }
            }
        }

        Ok(0 as usize)
    }

    // NOTE(amotta): A lot of the error handling in this function
    // could be simplified if there existed an automatic conversion
    // from io::Error to the wkw::Error.
    pub fn read_header(root: &Path) -> Result<Header> {
        let mut header_path = PathBuf::from(root);
        header_path.push(HEADER_FILE_NAME);

        let header_file_opt = File::open(header_path);

        if header_file_opt.is_err() {
            return Err("Could not open header file");
        }

        let mut header_file = header_file_opt.unwrap();

        let mut buf = [0u8; 16];
        let read_opt = header_file.read(&mut buf);

        if read_opt.is_err() || read_opt.unwrap() != buf.len() {
            return Err("Header file is too small")
        }

        Header::from_bytes(buf)
    }
}
