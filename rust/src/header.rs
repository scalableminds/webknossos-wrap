use crate::lz4;
use crate::result::Result;
use std::io::{Read, Write};
use std::{fs, mem, slice};

#[repr(C)]
#[derive(Debug)]
struct HeaderRaw {
    magic: [u8; 3],
    version: u8,
    per_dim_log2: u8,
    block_type: u8,
    voxel_type: u8,
    voxel_size: u8,
    data_offset: u64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockType {
    Raw,
    LZ4,
    LZ4HC,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoxelType {
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
}

impl VoxelType {
    pub fn size(&self) -> usize {
        match *self {
            VoxelType::U8 | VoxelType::I8 => 1,
            VoxelType::U16 | VoxelType::I16 => 2,
            VoxelType::U32 | VoxelType::I32 => 4,
            VoxelType::U64 | VoxelType::I64 => 8,
            VoxelType::F32 => 4,
            VoxelType::F64 => 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub version: u8,
    pub block_len_log2: u8,
    pub file_len_log2: u8,
    pub block_type: BlockType,
    pub voxel_type: VoxelType,
    pub voxel_size: u8,
    pub data_offset: u64,
    pub jump_table: Option<Box<[u64]>>,
}

impl Header {
    pub fn from_template(template: &Header) -> Header {
        let mut header = template.clone();
        header.init();
        header
    }

    pub fn compress(template: &Header) -> Header {
        let mut header = template.clone();
        header.block_type = BlockType::LZ4HC;
        header.init();
        header
    }

    fn init(&mut self) {
        self.data_offset = match self.version {
            1 => self.size_on_disk() as u64,
            2 => 0,
            _ => unreachable!(),
        };

        // initialize jump table
        self.jump_table = match self.block_type {
            BlockType::LZ4 | BlockType::LZ4HC => {
                let file_vol = self.file_vol() as usize;
                let jump_table = vec![0u64; file_vol];
                Some(jump_table.into_boxed_slice())
            }
            _ => None,
        };
    }

    pub fn size_on_disk(&self) -> usize {
        let header_len = 16;

        let jump_table_len = match self.block_type {
            BlockType::Raw => 0,
            BlockType::LZ4 | BlockType::LZ4HC => self.file_vol() as usize * mem::size_of::<u64>(),
        } as usize;

        header_len + jump_table_len
    }

    pub fn read_dataset_header(file: &mut fs::File) -> Result<Header> {
        Self::read(file)
    }

    pub fn read_file_header(file: &mut fs::File) -> Result<Header> {
        let mut header = Self::read(file)?;

        header.jump_table = match header.block_type {
            BlockType::LZ4 | BlockType::LZ4HC => Some(header.read_jump_table(file)?),
            _ => None,
        };

        Ok(header)
    }

    fn read(file: &mut fs::File) -> Result<Header> {
        let mut buf = [0u8; 16];

        match file.read_exact(&mut buf) {
            Err(_) => Err("Could not read raw header"),
            Ok(_) => Self::from_bytes(buf),
        }
    }

    pub fn write(&self, file: &mut fs::File) -> Result<()> {
        if file.write_all(&self.to_bytes()).is_err() {
            return Err("Could not write header");
        }

        match self.jump_table {
            Some(_) => self.write_jump_table(file),
            None => Ok(()),
        }
    }

    fn read_jump_table(&mut self, file: &mut fs::File) -> Result<Box<[u64]>> {
        // allocate jump table
        let block_count = self.file_vol() as usize;
        let mut jump_table = Vec::with_capacity(block_count);

        let result = unsafe {
            // slice of unsigned 64-bit integers
            jump_table.set_len(block_count);

            // slice of unsigned 8-bit integers
            let buf_u8_len = jump_table.len() * mem::size_of::<u64>();
            let buf_u8_ptr = jump_table.as_mut_ptr() as *mut u8;
            let buf_u8 = slice::from_raw_parts_mut(buf_u8_ptr, buf_u8_len);

            // read jump table
            file.read_exact(buf_u8)
        };

        match result {
            Ok(_) => Ok(jump_table.into_boxed_slice()),
            Err(_) => Err("Could not read jump table"),
        }
    }

    fn write_jump_table(&self, file: &mut fs::File) -> Result<()> {
        let jump_table = &*self.jump_table.as_ref().unwrap();

        let result = unsafe {
            let buf_u8_len = jump_table.len() * mem::size_of::<u64>();
            let buf_u8_ptr = jump_table.as_ptr() as *const u8;
            let buf_u8 = slice::from_raw_parts(buf_u8_ptr, buf_u8_len);

            // write jump table
            file.write_all(buf_u8)
        };

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not write jump table"),
        }
    }

    pub fn block_offset(&self, block_idx: u64) -> Result<u64> {
        if block_idx >= self.file_vol() {
            return Err("Block index out of bounds");
        }

        let offset = match self.block_type {
            BlockType::Raw => {
                let block_size = self.block_size() as u64;
                self.data_offset + block_idx * block_size
            }
            BlockType::LZ4 | BlockType::LZ4HC => {
                if block_idx == 0 {
                    self.data_offset
                } else {
                    let ref jump_table = *self.jump_table.as_ref().unwrap();
                    jump_table[block_idx as usize - 1]
                }
            }
        };

        Ok(offset)
    }

    pub fn block_size_on_disk(&self, block_idx: u64) -> Result<usize> {
        match self.block_type {
            BlockType::Raw => Ok(self.block_size() as usize),
            BlockType::LZ4 | BlockType::LZ4HC => {
                let jump_table = self.jump_table.as_ref().unwrap();

                if block_idx == 0 {
                    let block_size = jump_table[0] - self.data_offset;
                    Ok(block_size as usize)
                } else if block_idx < jump_table.len() as u64 {
                    let block_idx = block_idx as usize;
                    let block_size = jump_table[block_idx] - jump_table[block_idx - 1];
                    Ok(block_size as usize)
                } else {
                    Err("Block index out of bounds")
                }
            }
        }
    }

    pub fn total_size_of_blocks_on_disk(&self) -> usize {
        match self.block_type {
            BlockType::Raw => self.file_size(),
            BlockType::LZ4 | BlockType::LZ4HC => {
                let jump_table = self.jump_table.as_ref().unwrap();
                let end_of_last_block = jump_table[jump_table.len() - 1] as usize;
                let start_of_first_block = self.data_offset as usize;
                end_of_last_block - start_of_first_block
            }
        }
    }

    pub fn max_block_size_on_disk(&self) -> usize {
        let block_size = self.block_size();

        match self.block_type {
            BlockType::Raw => block_size,
            BlockType::LZ4 | BlockType::LZ4HC => lz4::compress_bound(block_size),
        }
    }

    pub fn voxel_type_size(&self) -> usize {
        self.voxel_type.size()
    }

    pub fn num_channels(&self) -> usize {
        let voxel_size = self.voxel_size as usize;
        assert!(voxel_size % self.voxel_type_size() == 0);
        voxel_size / self.voxel_type_size()
    }

    pub fn is_multi_channel(&self) -> bool {
        self.voxel_size as usize > self.voxel_type_size()
    }

    fn from_bytes(buf: [u8; 16]) -> Result<Header> {
        let raw: HeaderRaw = unsafe { mem::transmute(buf) };

        if &raw.magic != "WKW".as_bytes() {
            return Err("Sequence of magic bytes is invalid");
        }

        if raw.version < 1 || raw.version > 2 {
            return Err("Version number is invalid");
        }

        let block_len_log2 = raw.per_dim_log2 & 0x0f;
        let file_len_log2 = raw.per_dim_log2 >> 4;

        let block_type = match raw.block_type {
            1 => BlockType::Raw,
            2 => BlockType::LZ4,
            3 => BlockType::LZ4HC,
            _ => return Err("Block type is invalid"),
        };

        let voxel_type = match raw.voxel_type {
            1 => VoxelType::U8,
            2 => VoxelType::U16,
            3 => VoxelType::U32,
            4 => VoxelType::U64,
            5 => VoxelType::F32,
            6 => VoxelType::F64,
            7 => VoxelType::I8,
            8 => VoxelType::I16,
            9 => VoxelType::I32,
            10 => VoxelType::I64,
            _ => return Err("Voxel type is invalid"),
        };

        Ok(Header {
            version: raw.version,
            block_len_log2: block_len_log2,
            file_len_log2: file_len_log2,
            block_type: block_type,
            voxel_type: voxel_type,
            voxel_size: raw.voxel_size,
            data_offset: raw.data_offset,
            jump_table: None,
        })
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let per_dim_log2 = (self.file_len_log2 << 4) | (self.block_len_log2 & 0x0f);

        let mut raw = HeaderRaw {
            magic: [0u8; 3],
            version: self.version,
            per_dim_log2: per_dim_log2,
            block_type: 1u8 + self.block_type as u8,
            voxel_type: 1u8 + self.voxel_type as u8,
            voxel_size: self.voxel_size,
            data_offset: self.data_offset,
        };

        // set magic bytes
        raw.magic.copy_from_slice("WKW".as_bytes());

        // convert to bytes
        unsafe { mem::transmute::<HeaderRaw, [u8; 16]>(raw) }
    }

    pub fn block_len(&self) -> u16 {
        1u16 << self.block_len_log2
    }
    pub fn block_vol(&self) -> u64 {
        1u64 << (3 * self.block_len_log2)
    }
    pub fn block_size(&self) -> usize {
        self.voxel_size as usize * self.block_vol() as usize
    }

    pub fn file_len(&self) -> u16 {
        1u16 << self.file_len_log2
    }
    pub fn file_vol(&self) -> u64 {
        1u64 << (3 * self.file_len_log2)
    }

    pub fn file_len_vx_log2(&self) -> u8 {
        self.file_len_log2 + self.block_len_log2
    }
    pub fn file_len_vx(&self) -> u32 {
        1u32 << self.file_len_vx_log2() as u32
    }
    pub fn file_vol_vx(&self) -> u64 {
        1u64 << (3 * self.file_len_vx_log2() as u64)
    }
    pub fn file_size(&self) -> usize {
        self.voxel_size as usize * self.file_vol_vx() as usize
    }
}
