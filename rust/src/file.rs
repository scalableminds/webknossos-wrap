use crate::lz4;
use crate::{BlockType, Box3, Dataset, Header, Iter, Mat, Morton, Result, Vec3};
use std::io::{Read, Seek, SeekFrom, Write};
use std::{fs, io, path};

#[derive(Debug)]
pub struct File {
    file: fs::File,
    header: Header,
    block_idx: Option<u64>,
    block_buf: Option<Box<[u8]>>,
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
            block_buf,
            block_idx: None,
        }
    }

    pub fn open(dataset_header: &Header, path: &path::Path) -> Result<File> {
        let mut file = fs::File::open(path).or(Err("Could not open WKW file"))?;

        // NOTE(amotta): This is currently only really needed for WKW files with LZ4 or LZ4HC block
        // types. For raw block types we could just reuse the dataset header and thus save at least
        // one seek and read operation.
        Self::seek_header(dataset_header, &mut file)?;
        let header = Header::read_file_header(&mut file)?;

        Ok(Self::new(file, header))
    }

    pub(crate) fn open_or_create(
        dataset_header: &Header,
        path: &path::Path,
    ) -> Result<(bool, File)> {
        // create parent directory, if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).or(Err("Could not create parent directory"))?;
        }

        let mut open_opts = fs::OpenOptions::new();
        open_opts.read(true).write(true);
        let mut create_opts = fs::OpenOptions::new();
        create_opts.read(true).write(true).create_new(true);

        // try to create file
        let (created, file) = match create_opts.open(path) {
            Ok(file) => Ok((true, file)),
            Err(err) => match err.kind() {
                // if file already exists, we just open it
                io::ErrorKind::AlreadyExists => match open_opts.open(path) {
                    Ok(file) => Ok((false, file)),
                    Err(_) => Err("Could not open file"),
                },
                _ => Err("Could not create file"),
            },
        }?;

        // create structure
        let header = Header::from_template(dataset_header);
        let mut file = Self::new(file, header);

        if created && file.header.block_type == BlockType::Raw {
            file.truncate()?;
            file.write_header()?;
        }

        Ok((created, file))
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
        let mut buf = vec![0u8; self.header.block_size()];
        let mut src_mat = Mat::new(
            buf.as_mut_slice(),
            Vec3::from(1u32 << block_len_log2),
            self.header.voxel_size as usize,
            self.header.voxel_type,
        )?;

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

            // NOTE(amotta): The current approach reads each block into a temporary buffer /
            // matrix, which is then (possibly only partially) copied to the destination buffer/
            // matrix. This (currently) only really needed for LZ4 compressed blocks.

            // read data
            self.seek_block(cur_block_idx)?;
            self.read_block(src_mat.as_mut_slice())?;

            // copy data
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
        let mut buf = vec![0u8; self.header.block_size()];
        let mut buf_mat = Mat::new(
            buf.as_mut_slice(),
            Vec3::from(1u32 << block_len_log2),
            self.header.voxel_size as usize,
            self.header.voxel_type,
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
                self.read_block(buf_mat.as_mut_slice())?;
            }

            let cur_src_box = cur_box - dst_pos + src_pos;
            let cur_dst_pos = cur_box.min() - cur_block_box.min();

            // fill / modify buffer
            buf_mat.copy_from(cur_dst_pos, src_mat, cur_src_box)?;

            // write data
            self.seek_block(cur_block_idx)?;
            self.write_block(buf_mat.as_slice())?;
        }

        if self.header.block_type == BlockType::LZ4 || self.header.block_type == BlockType::LZ4HC {
            self.truncate()?;
            self.write_header()?;
        }

        Ok(1 as usize)
    }

    pub fn compress(src_path: &path::Path, dst_path: &path::Path) -> Result<()> {
        let mut src_file = {
            // NOTE(amotta): Now that there exist multiple versions of the WKW on-disk file format,
            // it is no longer possible to compress a file without looking at the header file of
            // the dataset. Without the version number from the header file it's impossible to
            // tell whether the header of a WKW file is stored at the beginning or at the end.
            let dataset_root = src_path
                .ancestors()
                .nth(3)
                .ok_or("Could not derive dataset root")?;
            let dataset = Dataset::new(dataset_root)?;
            File::open(dataset.header(), src_path)?
        };

        // TODO(amotta): This is not a good idea... The header for the compressed WKW file should
        // instead be derived from the header of the compressed WKW dataset. This allows, e.g., to
        // compress into datasets with higher on-disk format version numbers.
        let dst_header = Header::compress(&src_file.header);

        // make sure that output path does not exist yet
        let mut dst_file = match File::open_or_create(&dst_header, dst_path)? {
            (false, _) => Err("Output file already exists"),
            (true, file) => Ok(file),
        }?;

        // prepare buffers and jump table
        let mut buf_vec = vec![0u8; src_file.header.block_size()];
        let buf = buf_vec.as_mut_slice();

        // prepare files
        src_file.seek_block(0)?;
        dst_file.seek_block(0)?;

        for _idx in 0..dst_header.file_vol() {
            src_file.read_block(buf)?;
            dst_file.write_block(buf)?;
        }

        // write header (with jump table)
        dst_file.truncate()?;
        dst_file.write_header()
    }

    fn truncate(&self) -> Result<()> {
        let header_size = self.header.size_on_disk();
        let body_size = self.header.total_size_of_blocks_on_disk();
        let truncated_size = header_size + body_size;

        self.file
            .set_len(truncated_size as u64)
            .map_err(|_| "Could not truncate file")
    }

    fn seek_header(dataset_header: &Header, file: &mut fs::File) -> Result<()> {
        let seek_to = match dataset_header.version {
            1 => SeekFrom::Start(0 as u64),
            2 => {
                let header_size = dataset_header.size_on_disk();
                SeekFrom::End(-(header_size as i64))
            }
            _ => unreachable!(),
        };

        match file.seek(seek_to) {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not seek header"),
        }
    }

    fn write_header(&mut self) -> Result<()> {
        Self::seek_header(&self.header, &mut self.file)?;
        self.header.write(&mut self.file)
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_idx = self.block_idx.unwrap();
        let read = match self.header.block_type {
            BlockType::Raw => self.read_block_raw(buf),
            BlockType::LZ4 | BlockType::LZ4HC => self.read_block_lz4(buf),
        }?;

        self.block_idx = Some(block_idx + 1);
        Ok(read)
    }

    fn write_block(&mut self, buf: &[u8]) -> Result<usize> {
        let block_idx = match self.block_idx {
            Some(block_idx) => block_idx,
            None => return Err("File is not block aligned"),
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
            Err(_) => Err("Could not read raw block"),
        }
    }

    fn write_block_raw(&mut self, buf: &[u8]) -> Result<usize> {
        match self.file.write_all(buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => Err("Could not write raw block"),
        }
    }

    fn write_block_lz4(&mut self, buf: &[u8]) -> Result<usize> {
        // compress data
        let buf_lz4 = self.block_buf.as_mut().unwrap();
        let len_lz4 = lz4::compress_hc(buf, buf_lz4)?;

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
        let jump_table = self.header.jump_table.as_mut().unwrap();
        jump_table[block_idx as usize] = jump_entry;

        Ok(len_lz4)
    }

    fn read_block_lz4(&mut self, buf: &mut [u8]) -> Result<usize> {
        let buf_lz4 = {
            let idx = self.block_idx.unwrap();
            let len = self.header.block_size_on_disk(idx).unwrap();
            let buf = self.block_buf.as_mut().unwrap();
            &mut buf[..len]
        };

        // read compressed block
        self.file
            .read_exact(buf_lz4)
            .or(Err("Error while reading LZ4 block"))?;

        // decompress block
        let raw_len = lz4::decompress_safe(buf_lz4, buf)?;

        match raw_len == buf.len() {
            true => Ok(raw_len),
            false => Err("Unexpected length after decompression"),
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
            Err(_) => Err("Could not seek block"),
            Ok(_) => {
                self.block_idx = Some(block_idx);
                Ok(block_idx)
            }
        }
    }
}
