use lz4;
use std::{fs, path};
use std::io::{Read, Seek, SeekFrom, Write};
use ::{Header, BlockType, Iter, Mat, Morton, Result, Vec3, Box3};

#[derive(Debug)]
pub struct File {
    file: fs::File,
    header: Header,
    block_idx: Option<u64>,
    block_buf: Option<Box<[u8]>>
}

impl File {
    fn new(file: fs::File, header: Header) -> File {
        let block_buf = match header.block_type {
            BlockType::LZ4 | BlockType::LZ4HC => {
                let buf_size = header.max_block_size_on_disk();
                let buf_vec = vec![0u8; buf_size];
                Some(buf_vec.into_boxed_slice())
            },
            _ => None
        };

        File {
            file: file,
            header: header,
            block_idx: None,
            block_buf: block_buf
        }
    }

    pub fn open(path: &path::Path) -> Result<File> {
        let mut file = match fs::File::open(path) {
            Ok(file) => file,
            Err(_) => return Err("Could not open file")
        };

        // read header
        let header = Header::read(&mut file)?;

        Ok(Self::new(file, header))
    }

    pub fn open_or_create(path: &path::Path, header: &Header) -> Result<File> {
        // create parent directory, if needed
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return Err("Could not create parent directory");
            }
        }

        // create parent directory, if needed
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return Err("Could not create parent directory");
            }
        }

        let mut open_opts = fs::OpenOptions::new();
        open_opts.read(true).write(true).create(true);

        let mut file = match open_opts.open(path) {
            Ok(file) => file,
            Err(_) => return Err("Could not open file")
        };

        // check if file was created
        let (header, created) = match Header::read(&mut file) {
            Ok(header) => (header, false),
            Err(_) => (Header::from_template(header), true)
        };

        // create structure
        let mut file = Self::new(file, header);

        if created {
            file.truncate()?;
            file.write_header()?;
        }

        Ok(file)
    }

    pub fn header(&self) -> &Header { &self.header }

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

    pub fn write_mat(&mut self, dst_pos: Vec3, src_mat: &Mat, src_pos: Vec3) -> Result<usize> {
        let file_len_vx = self.header.file_len_vx();
        let file_len_log2 = self.header.file_len_log2 as u32;
        let block_len_log2 = self.header.block_len_log2 as u32;

        let file_len_vx_vec = Vec3::from(file_len_vx);
        assert!(src_pos < file_len_vx_vec);

        let src_len = src_mat.shape();
        let dst_end = file_len_vx_vec.elem_min(src_len - src_pos + dst_pos);
        let dst_box = Box3::new(dst_pos, dst_end)?;

        // bounding box in boxes
        let dst_box_boxes = Box3::new(
            dst_box.min() >> file_len_log2,
           (dst_box.max() >> file_len_log2) + 1)?;

        // allocate buffer
        let buf_shape = Vec3::from(1u32 << block_len_log2);
        let voxel_size = self.header.voxel_size as usize;

        let mut buf = vec![0u8; self.header.block_size()];
        let mut buf_mat = Mat::new(buf.as_mut_slice(), buf_shape, voxel_size)?;

        let iter = Iter::new(file_len_log2, dst_box_boxes)?;
        for cur_block_idx in iter {
            // box for current block
            let cur_block_ids = Vec3::from(Morton::from(cur_block_idx));

            let cur_block_box = Box3::new(
                cur_block_ids << block_len_log2,
               (cur_block_ids + 1) << block_len_log2)?;
            let cur_box = cur_block_box.intersect(dst_box);

            let cur_dst_pos = cur_box.min() - cur_block_box.min();
            let cur_src_box = cur_box - dst_pos + src_pos;

            // fill buffer
            buf_mat.copy_from(cur_dst_pos, src_mat, cur_src_box)?;

            // write data
            self.seek_block(cur_block_idx)?;
            self.write_block(buf_mat.as_slice())?;
        }

        Ok(1 as usize)
    }

    fn truncate(&mut self) -> Result<()> {
        match self.header.block_type {
            BlockType::Raw => {
                let header_size = self.header.size_on_disk();
                let body_size = self.header.file_size();
                let size = header_size + body_size;

                match self.file.set_len(size as u64) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Could not truncate file")
                }
            },
            _ => Ok(())
        }
    }

    fn seek_header(&mut self) -> Result<()> {
        match self.file.seek(SeekFrom::Start(0)) {
            Ok(0) => Ok(()),
            _ => Err("Could not seek header"),
        }
    }

    fn write_header(&mut self) -> Result<()> {
        self.seek_header()?;
        self.header.write(&mut self.file)
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() != self.header.block_size() {
            return Err("Buffer has invalid size");
        }

        let block_idx = match self.block_idx {
            Some(block_idx) => block_idx,
            None => return Err("File is not block aligned")
        };

        let bytes_read = match self.header.block_type {
            BlockType::Raw => self.read_block_raw(buf)?,
            BlockType::LZ4 | BlockType::LZ4HC => self.read_block_lz4(buf)?
        };

        // advance block index
        self.block_idx = Some(block_idx + 1);

        Ok(bytes_read)
    }

    fn write_block(&mut self, buf: &[u8]) -> Result<usize> {
        let block_idx = match self.block_idx {
            Some(block_idx) => block_idx,
            None => return Err("File is not block aligned")
        };

        let bytes_written = match self.header.block_type {
            BlockType::Raw => self.write_block_raw(buf)?,
            BlockType::LZ4 | BlockType::LZ4HC => return Err("Unsupported")
        };

        // advance
        self.block_idx = Some(block_idx + 1);

        Ok(bytes_written)
    }

    fn read_block_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.file.read_exact(buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => {
                self.block_idx = None;
                Err("Could not read raw block")
            }
        }
    }

    fn write_block_raw(&mut self, buf: &[u8]) -> Result<usize> {
        match self.file.write_all(buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err("Could not write raw block")
        }
    }

    fn read_block_lz4(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_idx = self.block_idx.unwrap();
        let block_size_lz4 = self.header.block_size_on_disk(block_idx)?;
        let block_size_raw = self.header.block_size();

        let buf_lz4_orig = &mut *self.block_buf.as_mut().unwrap();
        let buf_lz4 = &mut buf_lz4_orig[..block_size_lz4];

        // read compressed block
        if self.file.read_exact(buf_lz4).is_err() {
            self.block_idx = None;
            return Err("Error while reading LZ4 block");
        }

        // decompress block
        let byte_written = lz4::decompress_safe(buf_lz4, buf)?;

        match byte_written == block_size_raw {
            true => Ok(byte_written),
            false => Err("Unexpected length after decompression")
        }
    }

    fn seek_block(&mut self, block_idx: u64) -> Result<u64> {
        if self.block_idx == Some(block_idx) {
            return Ok(block_idx)
        }

        // determine block offset
        let offset = self.header.block_offset(block_idx)?;

        // seek to byte offset
        match self.file.seek(SeekFrom::Start(offset)) {
            Err(_) => Err("Could not seek block"),
            Ok(_) => {
                self.block_idx = Some(block_idx);
                Ok(block_idx)
            }
        }
    }
}
