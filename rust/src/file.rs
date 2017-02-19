use std::fs;
use std::io::{Read, Seek, SeekFrom};

use header::Header;
use mat::Mat;
use morton::Morton;
use result::Result;
use vec::Vec;

#[derive(Debug)]
pub struct File<'a> {
    file: &'a fs::File,
    header: Header,
    block_idx: Option<u64>
}

impl<'a> File<'a> {
    pub fn new(file: &'a mut fs::File) -> Result<File> {
        let mut buf = [0u8; 16];
        if file.read(&mut buf).unwrap() != 16 {
            return Err("Could not read file header");
        }

        // create file
        let header = Header::from_bytes(buf)?;
        let mut wkw_file = File { file: file, header: header, block_idx: None };

        // seek to first block
        wkw_file.seek_block(0 as u64)?;

        Ok(wkw_file)
    }

    pub fn header(&'a self) -> &'a Header { &self.header }

    pub fn read_mat(&mut self, mat: &mut Mat, off: &Vec) -> Result<usize> {
        if !off.is_valid_offset() {
            return Err("Offset is invalid");
        }

        if !mat.shape().is_power_of_two()
        || !mat.shape().is_larger_equal_than(off) {
            return Err("Shape of matrix is invalid");
        }

        let block_side_len = self.header.voxels_per_block_dim as u32;
        let block_ids = off.clone() / block_side_len.into();
        let block_idx = u64::from(Morton::from(&block_ids));

        let blocks_per_dim = mat.shape().clone() / block_side_len.into();
        let block_count =
            blocks_per_dim.x as usize *
            blocks_per_dim.y as usize *
            blocks_per_dim.z as usize;

        // seek to start
        self.seek_block(block_idx)?;

        for cur_idx in 0..block_count {
            // read a block
            let mut buf = vec![0 as u8; self.header.block_size];
            self.read_block(buf.as_mut_slice())?;

            // build matrix arround buffer
            let buf_mat = Mat::new(
                buf.as_mut_slice(),
                Vec::from(self.header.voxels_per_block_dim as u32),
                self.header.voxel_size as usize).unwrap();

            // determine target position
            let cur_pos = Vec::from(Morton::from(cur_idx as u64));

            // copy to target
            mat.copy_from(&buf_mat, &cur_pos)?;
        }

        Ok(mat.as_slice().len())
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_size = self.header.block_size;

        if buf.len() != block_size {
            return Err("Buffer has invalid size");
        }

        if self.file.read(buf).unwrap() != block_size {
            return Err("Could not read whole block");
        }

        Ok(block_size)
    }

    fn seek_block(&mut self, block_idx: u64) -> Result<u64> {
        if self.block_idx == Some(block_idx) {
            return Ok(block_idx)
        }

        // calculate byte offset
        let block_size = self.header.block_size as u64;
        let offset = self.header.data_offset + block_idx * block_size;

        // seek to byte offset
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.block_idx = Some(block_idx);

        Ok(block_idx)
    }
}
