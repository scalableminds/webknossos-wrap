use std::mem;
use result::Result;

#[repr(C)]
struct HeaderRaw {
    magic: [u8; 3],
    version: u8,
    per_dim_log2: u8,
    block_type: u8,
    voxel_type: u8,
    voxel_size: u8,
    data_offset: u64
}

#[derive(Debug)]
pub enum BlockType { Raw, LZ4, LZ4HC }

#[derive(Debug)]
pub enum VoxelType { U8, U16, U32, U64, F32, F64 }

#[derive(Debug)]
pub struct Header {
    pub version: u8,
    pub blocks_per_file_dim: u16,
    pub voxels_per_block_dim: u16,
    pub block_type: BlockType,
    pub block_size: usize,
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

        let blocks_per_file_dim_log2 = raw.per_dim_log2 & 0x0f;
        let voxels_per_block_dim_log2 = raw.per_dim_log2 >> 4;

        let blocks_per_file_dim = (1 as u16) << blocks_per_file_dim_log2;
        let voxels_per_block_dim = (1 as u16) << voxels_per_block_dim_log2;

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

        // TODO: check voxel size
        let block_size = (1 as usize) << (3 * voxels_per_block_dim_log2);

        Ok(Header {
            version: raw.version,
            blocks_per_file_dim: blocks_per_file_dim,
            voxels_per_block_dim: voxels_per_block_dim,
            block_type: block_type,
            block_size: block_size,
            voxel_type: voxel_type,
            voxel_size: raw.voxel_size,
            data_offset: raw.data_offset
        })
    }
}
