use lz4;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use ::{Header, BlockType, Iter, Mat, Morton, Result, Vec3, Box3};

#[derive(Debug)]
pub struct File<'a> {
    file: &'a fs::File,
    header: Header,
    block_idx: Option<u64>,
    block_buf: Option<Box<[u8]>>
}

impl<'a> File<'a> {
    pub fn new(file: &'a mut fs::File) -> Result<File> {
        let header = Header::read(file)?;

        let block_buf = match header.block_type {
            BlockType::LZ4 | BlockType::LZ4HC => {
                let buf_size = header.max_block_size();
                let buf_vec = vec![0u8; buf_size];
                Some(buf_vec.into_boxed_slice())
            },
            _ => None
        };

        let wkw_file = File {
            file: file,
            header: header,
            block_idx: None,
            block_buf: block_buf
        };

        Ok(wkw_file)
    }

    pub fn header(&'a self) -> &'a Header { &self.header }

    pub fn read_mat(&mut self, src_pos: Vec3, dst_mat: &mut Mat, dst_pos: Vec3) -> Result<usize> {
        let file_len_vx = self.header.file_len_vx();
        let file_len_log2 = self.header.file_len_log2 as u32;
        let block_len_log2 = self.header.block_len_log2 as u32;

        let file_len_vx_vec = Vec3::from(file_len_vx);
        assert!(src_pos < file_len_vx_vec);

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
        let buf_shape = Vec3::from(1u32 << block_len_log2);
        let mut buf_vec = vec![0u8; block_size];
        let mut buf = buf_vec.as_mut_slice();

        let iter = Iter::new(file_len_log2, src_box_boxes)?;
        for cur_block_idx in iter {
            // box for current block
            let cur_block_ids = Vec3::from(Morton::from(cur_block_idx));

            let cur_block_box = Box3::new(
                cur_block_ids << block_len_log2,
               (cur_block_ids + 1) << block_len_log2)?;
            let cur_box = cur_block_box.intersect(src_box);

            // source and destination offsets
            let cur_dst_pos = cur_box.min() - src_pos + dst_pos;
            let cur_src_box = cur_box - cur_block_box.min();

            // read data
            self.seek_block(cur_block_idx)?;
            self.read_block(buf)?;

            // copy data
            let src_mat = Mat::new(buf, buf_shape, voxel_size)?;
            dst_mat.copy_from(cur_dst_pos, &src_mat, cur_src_box)?;
        }

        Ok(1 as usize)
    }

    fn block_size(&self, block_idx: u64) -> Result<usize> {
            match self.header.block_type {
                BlockType::Raw => Ok(self.header.block_size() as usize),
                BlockType::LZ4 | BlockType::LZ4HC => {
                    let ref jump_table =
                        *self.header.jump_table.as_ref().unwrap();

                    if block_idx == 0 {
                        let block_size = jump_table[0] - self.header.data_offset;
                        Ok(block_size as usize)
                    } else if block_idx < self.header.file_vol() {
                        let block_idx = block_idx as usize;
                        let block_size = jump_table[block_idx] - jump_table[block_idx - 1];
                        Ok(block_size as usize)
                    } else {
                        Err("Block index out of bounds")
                    }
                }
            }
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() != self.header.block_size() {
            return Err("Buffer has invalid size");
        }

        if self.block_idx.is_none() {
            return Err("File is not block aligned");
        }

        let result = match self.header.block_type {
            BlockType::Raw => self.read_block_raw(buf),
            BlockType::LZ4 | BlockType::LZ4HC => self.read_block_lz4(buf)
        };

        // advance to next block
        self.block_idx.map(|idx| idx + 1);

        result
    }

    fn read_block_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes_read = match self.file.read(buf) {
            Err(_) => return Err("Error while reading raw block"),
            Ok(bytes_read) => bytes_read
        };

        let block_idx = self.block_idx.unwrap();
        let block_size = self.block_size(block_idx)?;

        if bytes_read != block_size {
            self.block_idx = None;
            return Err("Could not read whole raw block");
        }

        Ok(bytes_read)
    }

    fn read_block_lz4(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_idx = self.block_idx.ok_or("Block index missing")?;
        let block_size_lz4 = self.block_size(block_idx)?;
        let block_size_raw = self.header.block_size();

        let buf_lz4_orig = &mut *self.block_buf.as_mut().ok_or("Block buffer missing")?;
        let buf_lz4 = &mut buf_lz4_orig[..block_size_lz4];

        // read compressed block
        if self.file.read_exact(buf_lz4).is_err() {
            self.block_idx = None;
            return Err("Error while reading LZ4 block");
        }

        // decompress block
        let byte_written = lz4::decompress_safe(buf_lz4, buf)?;

        if byte_written != block_size_raw {
            return Err("Decompression produced invalid length");
        }

        Ok(byte_written)
    }

    fn seek_block(&mut self, block_idx: u64) -> Result<u64> {
        if self.block_idx == Some(block_idx) {
            return Ok(block_idx)
        }

        // determine block offset
        let offset = self.block_offset(block_idx)?;

        // seek to byte offset
        match self.file.seek(SeekFrom::Start(offset)) {
            Err(_) => Err("Could not seek block"),
            Ok(_) => {
                self.block_idx = Some(block_idx);
                Ok(block_idx)
            }
        }
    }

    fn block_offset(&self, block_idx: u64) -> Result<u64> {
        if block_idx >= self.header.file_vol() {
            return Err("Block index out of bounds");
        }

        let offset = match self.header.block_type {
            BlockType::Raw => {
                let block_size = self.header.block_size() as u64;
                self.header.data_offset + block_idx * block_size
            },
            BlockType::LZ4 | BlockType::LZ4HC => {
                if block_idx == 0 {
                    self.header.data_offset
                } else {
                    let ref jump_table =
                        *self.header.jump_table.as_ref().ok_or("Jump table missing")?;
                    jump_table[block_idx as usize - 1]
                }
            }
        };

        Ok(offset)
    }
}
