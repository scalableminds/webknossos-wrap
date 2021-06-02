use lz4_binding;
use std::io::{Read, Seek, SeekFrom, Write};
use std::{fs, path};
use {BlockType, Box3, Header, Iter, Mat, Morton, Result, Vec3};

#[derive(Debug)]
pub struct File {
    file: fs::File,
    header: Header,
    block_idx: Option<u64>,
    disk_block_buf: Option<Box<[u8]>>,
}

impl File {
    fn new(file: fs::File, header: Header) -> File {
        let block_buf = match header.block_type {
            BlockType::LZ4 | BlockType::LZ4HC => {
                let buf_size = header.max_block_size_on_disk();
                let buf_vec = vec![0u8; buf_size];
                Some(buf_vec.into_boxed_slice())
            }
            _ => None,
        };

        File {
            file,
            header,
            block_idx: None,
            disk_block_buf: block_buf,
        }
    }

    pub fn open(path: &path::Path) -> Result<File> {
        let mut file =
            fs::File::open(path).or(Err(format!("Could not open WKW file {:?}", path)))?;
        let header = Header::read(&mut file)?;
        Ok(Self::new(file, header))
    }

    pub(crate) fn open_or_create(path: &path::Path, header: &Header) -> Result<File> {
        // create parent directory, if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).or(Err(format!(
                "Could not create parent directory {:?}",
                parent
            )))?;
        }

        let mut open_opts = fs::OpenOptions::new();
        open_opts.read(true).write(true).create(true);

        let mut file = open_opts
            .open(path)
            .or(Err(format!("Could not open file {:?}", path)))?;

        // check if file was created
        let (header, created) = match Header::read(&mut file) {
            Ok(header) => (header, false),
            Err(_) => (Header::from_template(header), true),
        };

        // create structure
        let mut file = Self::new(file, header);

        if created {
            file.truncate()?;
            file.write_header()?;
        }

        Ok(file)
    }

    pub(crate) fn read_mat(
        &mut self,
        src_pos: Vec3,
        dst_mat: &mut Mat,
        dst_pos: Vec3,
    ) -> Result<usize> {
        let file_len_vx = self.header.file_len_vx();
        let file_len_log2 = self.header.file_len_log2 as u32;
        let block_len_log2 = self.header.block_len_log2 as u32;

        let file_len_vx_vec = Vec3::from(file_len_vx);
        assert!(src_pos < file_len_vx_vec);

        let dst_len = dst_mat.shape;
        let src_end = file_len_vx_vec.elem_min(src_pos + dst_len - dst_pos);

        // bounding boxes
        let src_box = Box3::new(src_pos, src_end)?;
        let src_box_boxes = Box3::new(
            src_box.min() >> block_len_log2,
            ((src_box.max() - 1) >> block_len_log2) + 1,
        )?;

        // allocate buffer
        let block_size = self.header.block_size();
        let voxel_size = self.header.voxel_size as usize;
        let voxel_type = self.header.voxel_type;

        let buf_shape = Vec3::from(1u32 << block_len_log2);
        let mut buf_vec = vec![0u8; block_size];
        let buf = buf_vec.as_mut_slice();

        let iter = Iter::new(file_len_log2, src_box_boxes)?;
        for cur_block_idx in iter {
            // box for current block
            let cur_block_ids = Vec3::from(Morton::from(cur_block_idx));

            let cur_block_box = Box3::new(
                cur_block_ids << block_len_log2,
                (cur_block_ids + 1) << block_len_log2,
            )?;
            let cur_box = cur_block_box.intersect(src_box);

            // source and destination offsets
            let cur_dst_pos = cur_box.min() - src_pos + dst_pos;
            let cur_src_box = cur_box - cur_block_box.min();

            // read data
            self.seek_block(cur_block_idx)?;
            self.read_block(buf)?;

            // copy data
            let src_mat = Mat::new(buf, buf_shape, voxel_size, voxel_type, false)?;
            dst_mat.copy_from(cur_dst_pos, &src_mat, cur_src_box)?;
        }

        Ok(1 as usize)
    }

    pub(crate) fn write_mat(
        &mut self,
        dst_pos: Vec3,
        src_mat: &Mat,
        src_pos: Vec3,
    ) -> Result<usize> {
        let block_len_log2 = self.header.block_len_log2 as u32;

        let dst_end =
            Vec3::from(self.header.file_len_vx()).elem_min(src_mat.shape - src_pos + dst_pos);
        let dst_box = Box3::new(dst_pos, dst_end)?;

        // bounding boxes
        let dst_box_boxes = Box3::new(
            dst_box.min() >> block_len_log2,
            ((dst_box.max() - 1) >> block_len_log2) + 1,
        )?;

        // build buffer matrix
        let mut src_block_buf = vec![0u8; self.header.block_size()];
        let mut src_block_buf_mat = Mat::new(
            src_block_buf.as_mut_slice(),
            Vec3::from(1u32 << block_len_log2),
            self.header.voxel_size as usize,
            self.header.voxel_type,
            false,
        )?;

        // build second buffer
        let mut c_to_fortran_buf = vec![0u8; self.header.block_size()];
        let mut c_to_fortran_buf_mat = Mat::new(
            c_to_fortran_buf.as_mut_slice(),
            Vec3::from(1u32 << block_len_log2),
            self.header.voxel_size as usize,
            self.header.voxel_type,
            true,
        )?;

        // build Morton-order iterator
        let iter = Iter::new(self.header.file_len_log2 as u32, dst_box_boxes)?;

        for cur_block_idx in iter {
            // box for current block
            let cur_block_ids = Vec3::from(Morton::from(cur_block_idx));

            let cur_block_box = Box3::new(
                cur_block_ids << block_len_log2,
                (cur_block_ids + 1) << block_len_log2,
            )?;
            let cur_box = cur_block_box.intersect(dst_box);

            if cur_box != cur_block_box {
                // reuse existing data
                self.seek_block(cur_block_idx)?;
                self.read_block(src_block_buf_mat.as_mut_slice())?;
            }

            let cur_src_box = cur_box - dst_pos + src_pos;
            let cur_dst_pos = cur_box.min() - cur_block_box.min();

            // fill / modify buffer
            src_block_buf_mat.copy_from_order_agnostic(
                cur_dst_pos,
                src_mat,
                cur_src_box,
                &mut c_to_fortran_buf_mat,
            )?;

            self.seek_block(cur_block_idx)?;

            // write in fortran order
            self.write_block(src_block_buf_mat.as_slice())?;
        }

        if self.header.block_type == BlockType::LZ4 || self.header.block_type == BlockType::LZ4HC {
            // Update jump table
            self.write_header()?;
            self.truncate()?;
        }

        Ok(1 as usize)
    }

    pub fn compress(&mut self, path: &path::Path) -> Result<()> {
        // prepare header
        let header = Header::compress(&self.header);

        // make sure that output path does not exist yet
        let mut file = match path.exists() {
            true => return Err(format!("Output file {:?} already exists", path)),
            false => Self::open_or_create(path, &header)?,
        };

        // prepare buffers and jump table
        let mut buf_vec = vec![0u8; self.header.block_size()];
        let buf = buf_vec.as_mut_slice();

        // prepare files
        self.seek_block(0)?;
        file.seek_block(0)?;

        for _idx in 0..header.file_vol() {
            self.read_block(buf)?;
            file.write_block(buf)?;
        }

        // write header (with jump table)
        file.write_header()
    }

    fn truncate(&self) -> Result<()> {
        let truncated_size = match self.header.block_type {
            BlockType::Raw => {
                let header_size = self.header.size_on_disk();
                let body_size = self.header.file_size();
                let size = header_size + body_size;
                size as u64
            }
            BlockType::LZ4 | BlockType::LZ4HC => {
                let last_block_idx = self.header.file_vol() - 1;
                let jump_table = self.header.jump_table.as_ref().unwrap();
                jump_table[last_block_idx as usize]
            }
        };

        self.file
            .set_len(truncated_size)
            .map_err(|_| String::from("Could not truncate file"))
    }

    fn seek_header(&mut self) -> Result<()> {
        match self.file.seek(SeekFrom::Start(0)) {
            Ok(0) => Ok(()),
            _ => Err(String::from("Could not seek header")),
        }
    }

    fn write_header(&mut self) -> Result<()> {
        self.seek_header()?;
        self.header.write(&mut self.file)
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() != self.header.block_size() {
            return Err(String::from("Buffer has invalid size"));
        }

        let block_idx = match self.block_idx {
            Some(block_idx) => block_idx,
            None => return Err(String::from("File is not block aligned")),
        };

        let result = match self.header.block_type {
            BlockType::Raw => self.read_block_raw(buf),
            BlockType::LZ4 | BlockType::LZ4HC => self.read_block_lz4(buf),
        };

        match result {
            Ok(_) => self.block_idx = Some(block_idx + 1),
            Err(_) => self.block_idx = None,
        };

        result
    }

    fn write_block(&mut self, buf: &[u8]) -> Result<usize> {
        let block_idx = match self.block_idx {
            Some(block_idx) => block_idx,
            None => return Err(String::from("File is not block aligned")),
        };

        let result = match self.header.block_type {
            BlockType::Raw => self.write_block_raw(buf),
            BlockType::LZ4 | BlockType::LZ4HC => self.write_block_lz4(buf),
        };

        // advance
        match result {
            Ok(_) => self.block_idx = Some(block_idx + 1),
            Err(_) => self.block_idx = None,
        };

        result
    }

    fn read_block_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.file.read_exact(buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err(String::from("Could not read raw block")),
        }
    }

    fn write_block_raw(&mut self, buf: &[u8]) -> Result<usize> {
        match self.file.write_all(buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err(String::from("Could not write raw block")),
        }
    }

    fn write_block_lz4(&mut self, buf: &[u8]) -> Result<usize> {
        // compress data
        let mut buf_lz4 = &mut *self.disk_block_buf.as_mut().unwrap();
        let len_lz4 = lz4_binding::compress_hc(buf, &mut buf_lz4)?;

        // write data
        self.file
            .write_all(&buf_lz4[..len_lz4])
            .or(Err("Could not write LZ4 block"))?;

        // update jump table
        let jump_entry = self
            .file
            .seek(SeekFrom::Current(0))
            .or(Err("Could not determine jump entry"))?;

        let block_idx = self.block_idx.unwrap();
        let jump_table = &mut *self.header.jump_table.as_mut().unwrap();
        jump_table[block_idx as usize] = jump_entry;

        Ok(len_lz4)
    }

    fn read_block_lz4(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_idx = self.block_idx.unwrap();
        let block_size_lz4 = self.header.block_size_on_disk(block_idx)?;
        let block_size_raw = self.header.block_size();

        let buf_lz4_orig = &mut *self.disk_block_buf.as_mut().unwrap();
        let buf_lz4 = &mut buf_lz4_orig[..block_size_lz4];

        // read compressed block
        self.file
            .read_exact(buf_lz4)
            .or(Err("Error while reading LZ4 block"))?;

        // decompress block
        let byte_written = lz4_binding::decompress_safe(buf_lz4, buf)?;

        match byte_written == block_size_raw {
            true => Ok(byte_written),
            false => Err(String::from("Unexpected length after decompression")),
        }
    }

    fn seek_block(&mut self, block_idx: u64) -> Result<u64> {
        if self.block_idx == Some(block_idx) {
            return Ok(block_idx);
        }

        // determine block offset
        let offset = self.header.block_offset(block_idx)?;

        // seek to byte offset
        match self.file.seek(SeekFrom::Start(offset)) {
            Err(_) => Err(String::from("Could not seek block")),
            Ok(_) => {
                self.block_idx = Some(block_idx);
                Ok(block_idx)
            }
        }
    }
}
