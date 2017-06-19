use std::fs;
use std::io::{Read, Seek, SeekFrom};
use ::{Header, Iter, Mat, Morton, Result, Vec3, Box3};

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

    pub fn aligned_blocks(&self, mat: &Mat, off: &Vec3) -> Option<(u64, u64)> {
        if self.is_aligned(mat, off) {
            let block_len_log2 = self.header.block_len_log2 as u32;

            let block_off_vec = off.clone() >> block_len_log2;
            let block_off = u64::from(Morton::from(&block_off_vec));

            let block_side_len = mat.shape().x >> block_len_log2;
            let block_count = block_side_len * block_side_len * block_side_len;

            Some((block_off, block_count as u64))
        } else {
            None
        }
    }

    pub fn is_aligned(&self, mat: &Mat, off: &Vec3) -> bool {
        mat.shape().is_cube_diagonal()
        && mat.shape().x.is_power_of_two()
        && mat.shape().x >= self.header.block_len() as u32
        && off.is_multiple_of(mat.shape())
    }

    pub fn read_aligned_mat(
        &mut self,
        block_off: u64,
        block_count: u64,
        mat: &mut Mat
    ) -> Result<usize> {

        let mut buf = vec![0 as u8; self.header.block_size()];
        let buf_shape = Vec3::from(self.header.block_len() as u32);
        let buf_width = self.header.voxel_size as usize;

        self.seek_block(block_off)?;

        for cur_idx in 0..block_count {
            self.read_block(buf.as_mut_slice())?;

            // determine target position
            let cur_blk_ids = Vec3::from(Morton::from(cur_idx as u64));
            let cur_pos = cur_blk_ids << self.header.block_len_log2 as u32;

            // copy to target
            let buf_mat = Mat::new(buf.as_mut_slice(), buf_shape, buf_width)?;
            mat.copy_all_from(cur_pos, &buf_mat)?;
        }

        Ok(mat.as_slice().len())
    }

    pub fn read_mat(&mut self, src_pos: Vec3, dst_mat: &mut Mat, dst_pos: Vec3) -> Result<usize> {
        let file_len_vx = self.header.file_len_vx();
        let file_len_log2 = self.header.file_len_log2 as u32;
        let block_len_log2 = self.header.block_len_log2 as u32;

        let block_len_vec = Vec3::from(1u32) << block_len_log2;
        let file_len_vx_vec = Vec3::from(file_len_vx);
        debug_assert!(src_pos < file_len_vx_vec);

        let dst_len = dst_mat.shape();
        let src_end = file_len_vx_vec.elem_min(src_pos + dst_len - dst_pos);
        let src_box = Box3::new(src_pos, src_end)?;

        // bounding box in boxes
        let src_box_boxes = Box3::new(
            src_box.min() >> file_len_log2,
           (src_box.max() >> file_len_log2) + 1)?;

        // allocate buffer
        let block_size = self.header.block_size();
        let voxel_size = self.header.voxel_size as usize;
        let buf_shape = Vec3::from(1u32) << block_len_log2;
        let mut buf_vec = vec![0u8; block_size];
        let mut buf = buf_vec.as_mut_slice();

        let iter = Iter::new(file_len_log2, src_box_boxes)?;
        for cur_idx in iter {
            // box for current block
            let cur_idx_vec = Vec3::from(Morton::from(cur_idx));
            let cur_block_box = Box3::new(cur_idx_vec, cur_idx_vec + 1)? << block_len_log2;

            let cur_box = Box3::new(
                cur_block_box.min().elem_max(src_pos),
                cur_block_box.max().elem_min(src_end)
            )?;

            // source and destination offsets
            let cur_dst_pos = cur_box.min() - src_pos + dst_pos;
            let cur_src_box = cur_box - cur_block_box.min();

            // read data
            self.seek_block(cur_idx)?;
            self.read_block(buf)?;

            // copy data
            let src_mat = Mat::new(buf, block_len_vec, voxel_size)?;
            dst_mat.copy_from(cur_dst_pos, &src_mat, cur_src_box);
        }

        Ok(1 as usize)
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_size = self.header.block_size();

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
        let block_size = self.header.block_size() as u64;
        let offset = self.header.data_offset + block_idx * block_size;

        // seek to byte offset
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.block_idx = Some(block_idx);

        Ok(block_idx)
    }
}
