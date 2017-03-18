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

    pub fn aligned_blocks(&self, mat: &Mat, off: &Vec) -> Option<(u64, u64)> {
        if self.is_aligned(mat, off) {
            let block_side_len = self.header.voxels_per_block_dim as u32;

            let block_off_vec = off.clone() / block_side_len;
            let block_off = u64::from(Morton::from(&block_off_vec));

            let block_side_len = mat.shape().x / block_side_len;
            let block_count = block_side_len * block_side_len * block_side_len;

            Some((block_off, block_count as u64))
        } else {
            None
        }
    }

    pub fn is_aligned(&self, mat: &Mat, off: &Vec) -> bool {
        mat.shape().is_cube_diagonal()
        && mat.shape().x.is_power_of_two()
        && mat.shape().x >= self.header.voxels_per_block_dim as u32
        && off.is_multiple_of(mat.shape())
    }

    pub fn read_mat(&mut self, mat: &mut Mat, off: &Vec) -> Result<usize> {
        match self.aligned_blocks(mat, off) {
            Some((block_off, block_count)) => self.read_aligned_mat(block_off, block_count, mat),
            None => Err("This library does not yet support unaligned reads")
        }
    }

    pub fn read_aligned_mat(
        &mut self,
        block_off: u64,
        block_count: u64,
        mat: &mut Mat
    ) -> Result<usize> {
        self.seek_block(block_off)?;

        let bytes_per_blk = self.header.block_size;
        let vx_per_blk_dim = self.header.voxels_per_block_dim;

        for cur_idx in 0..block_count {
            // read a block
            let mut buf = vec![0 as u8; bytes_per_blk];
            self.read_block(buf.as_mut_slice())?;

            // build matrix arround buffer
            let buf_mat = Mat::new(
                buf.as_mut_slice(),
                Vec::from(vx_per_blk_dim as u32),
                self.header.voxel_size as usize).unwrap();

            // determine target position
            let cur_blk_ids = Vec::from(Morton::from(cur_idx as u64));
            let cur_pos = cur_blk_ids * vx_per_blk_dim as u32;

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
