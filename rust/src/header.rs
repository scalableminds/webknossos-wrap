use std::mem;
use result::Result;

#[repr(C)]
#[derive(Debug)]
struct HeaderRaw {
    magic: [u8; 3],
    version: u8,
    per_dim_log2: u8,
    block_type: u8,
    voxel_type: u8,
    voxel_size: u8,
    data_offset: u64
}

#[derive(Copy, Clone, Debug)]
pub enum BlockType { Raw, LZ4, LZ4HC }

#[derive(Copy, Clone, Debug)]
pub enum VoxelType { U8, U16, U32, U64, F32, F64 }

#[derive(Copy, Clone, Debug)]
pub struct Header {
    pub version: u8,
    pub block_len_log2: u8,
    pub file_len_log2: u8,
    pub block_type: BlockType,
    pub voxel_type: VoxelType,
    pub voxel_size: u8,
    pub data_offset: u64
}

impl Header {
    pub fn from_bytes(buf: [u8; 16]) -> Result<Header> {
        let raw: HeaderRaw = unsafe { mem::transmute(buf) };

        if &raw.magic != "WKW".as_bytes() {
            return Err("Sequence of magic bytes is invalid");
        }

        if raw.version != 1 {
            return Err("Version number is invalid");
        }

        let block_len_log2 = raw.per_dim_log2 & 0x0f;
        let file_len_log2 = raw.per_dim_log2 >> 4;

        let block_type = match raw.block_type {
            1 => BlockType::Raw,
            2 => BlockType::LZ4,
            3 => BlockType::LZ4HC,
            _ => return Err("Block type is invalid")
        };

        let voxel_type = match raw.voxel_type {
            1 => VoxelType::U8,
            2 => VoxelType::U16,
            3 => VoxelType::U32,
            4 => VoxelType::U64,
            5 => VoxelType::F32,
            6 => VoxelType::F64,
            _ => return Err("Voxel type is invalid")
        };

        Ok(Header {
            version: raw.version,
            block_len_log2: block_len_log2,
            file_len_log2: file_len_log2,
            block_type: block_type,
            voxel_type: voxel_type,
            voxel_size: raw.voxel_size,
            data_offset: raw.data_offset
        })
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let per_dim_log2 = (self.file_len_log2 << 4)
                         | (self.block_len_log2 & 0x0f);

        let mut raw = HeaderRaw {
            magic: [0u8; 3],
            version: self.version,
            per_dim_log2: per_dim_log2,
            block_type: 1u8 + self.block_type as u8,
            voxel_type: 1u8 + self.voxel_type as u8,
            voxel_size: self.voxel_size,
            data_offset: self.data_offset
        };

        // set magic bytes
        raw.magic.copy_from_slice("WKW".as_bytes());

        // convert to bytes
        unsafe { mem::transmute::<HeaderRaw, [u8; 16]>(raw) }
    }

    pub fn block_len(&self) -> u16 { 1u16 << self.block_len_log2 }
    pub fn block_vol(&self) -> u64 { 1u64 << (3 * self.block_len_log2) }
    pub fn block_size(&self) -> usize { self.voxel_size as usize * self.block_vol() as usize }
}
