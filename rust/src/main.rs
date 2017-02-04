use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use std::io;
use std::mem;
use std::result;

type Result<T> = result::Result<T, &'static str>;

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

#[derive(Debug)]
enum BlockType { Raw, LZ4, LZ4HC }

#[derive(Debug)]
enum VoxelType { U8, U16, U32, U64, F32, F64 }

#[derive(Debug)]
struct Header {
    version: u8,
    blocks_per_file_dim: u16,
    voxels_per_block_dim: u16,
    block_type: BlockType,
    block_size: usize,
    voxel_type: VoxelType,
    voxel_size: u8,
    data_offset: u64
}

#[derive(Debug)]
struct WkwFile<'a> {
    file: &'a File,
    header: Header
}

impl<'a> WkwFile<'a> {
    fn from_file(file: &'a mut File) -> Result<WkwFile> {
        let header = wkw_read_header(file).unwrap();

        Ok(WkwFile {
            file: file,
            header: header
        })
    }

    fn read_block(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_size = self.header.block_size;

        if buf.len() < block_size {
            return Err("Buffer capacity is too small");
        }

        if self.file.read(buf).unwrap() < block_size {
            return Err("Could not read whole block");
        }

        Ok(block_size)
    }

    fn seek_block(&mut self, block_idx: u64) -> Result<u64> {
        let block_size = self.header.block_size as u64;
        let offset = self.header.data_offset + block_idx * block_size;

        Ok(self.file.seek(SeekFrom::Start(offset)).unwrap())
    }
}

fn main() {
    let wkw_path = "/home/amotta/Desktop/x000001_y000001_z000001.wkw";

    let mut file = File::open(wkw_path).unwrap();
    let mut wkw_file = WkwFile::from_file(&mut file).unwrap();

    // read a block
    let mut buf = vec![0 as u8; wkw_file.header.block_size];
    wkw_file.seek_block(1234).unwrap();
    wkw_file.read_block(buf.as_mut_slice()).unwrap();

    println!("{:#?}", wkw_file);
}

fn wkw_read_header_raw(file: &mut File) -> io::Result<HeaderRaw> {
    let buf = &mut [0 as u8; 16];
    let bytes_read = try!(file.read(buf));
    assert_eq!(bytes_read, 16);

    unsafe {
        Ok(mem::transmute_copy(buf))
    }
}

fn wkw_read_header(file: &mut File) -> Result<Header> {
    let raw = wkw_read_header_raw(file).unwrap();

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

