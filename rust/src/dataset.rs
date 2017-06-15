extern crate walkdir;
use dataset::walkdir::{DirEntry, WalkDir, WalkDirIterator};

use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::fs;

use file::File;
use header::Header;
use result::Result;

#[derive(Debug)]
pub struct Dataset<'a> {
    root: &'a OsStr,
    header: Header
}

static HEADER_FILE_NAME: &'static str = "header.wkw";

fn is_wkw_file(entry: &DirEntry) -> bool {
    let is_wkw = entry.file_name()
                      .to_str()
                      .map(|s| s.ends_with(".wkw"))
                      .unwrap_or(false);
    let is_file = entry.metadata()
                       .map(|m| m.is_file())
                       .unwrap_or(false);

    is_wkw && is_file
}

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

    pub fn recover_header(&self) -> Result<()> {
        // find an arbitrary .wkw file
        let mut walker = WalkDir::new(self.root)
                                 .min_depth(3).max_depth(3)
                                 .into_iter()
                                 .filter_entry(|e| is_wkw_file(e));

        let wkw_file_entry = match walker.next() {
            Some(Ok(s)) => s,
            Some(Err(_)) => return Err("Error in directory walk"),
            None => return Err("No .wkw files found")
        };

        // open wkw file
        let mut wkw_file_handle = fs::File::open(wkw_file_entry.path()).unwrap();
        let wkw_file = File::new(&mut wkw_file_handle).unwrap();

        // build header for meta file
        let mut wkw_header = wkw_file.header().clone();
        wkw_header.data_offset = 0;

        // convert to bytes
        let wkw_header_bytes = wkw_header.to_bytes();

        // build path to header file
        let mut header_file_path = PathBuf::from(self.root);
        header_file_path.push(HEADER_FILE_NAME);

        // write header
        let mut header_file_handle = fs::File::create(header_file_path).unwrap();
        header_file_handle.write(&wkw_header_bytes).unwrap();

        Ok(())
    }

    // NOTE(amotta): A lot of the error handling in this function
    // could be simplified if there existed an automatic conversion
    // from io::Error to the wkw::Error.
    pub fn read_header(root: &Path) -> Result<Header> {
        let mut header_path = PathBuf::from(root);
        header_path.push(HEADER_FILE_NAME);

        let header_file_opt = fs::File::open(header_path);

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
