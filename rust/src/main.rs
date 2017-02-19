use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ops::{Add, Div};
use std::option::Option;

use std::io;
use std::mem;
use std::ptr;
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

#[derive(Copy, Clone)]
struct WkwVec { x: u32, y: u32, z: u32 }

impl WkwVec {
    fn from_scalar(s: u32) -> WkwVec {
        WkwVec { x: s, y: s, z: s }
    }

    fn is_valid_offset(&self) -> bool {
        self.x == self.y &&
        self.x == self.z &&
        self.y == self.z &&
       (self.x == 0 || self.x.is_power_of_two())
    }

    fn is_power_of_two(&self) -> bool {
        self.x.is_power_of_two() &&
        self.y.is_power_of_two() &&
        self.z.is_power_of_two()
    }

    fn is_larger_equal_than(&self, other: &WkwVec) -> bool {
        self.x >= other.x &&
        self.y >= other.y &&
        self.z >= other.z
    }
}

impl From<u32> for WkwVec {
    fn from(s: u32) -> WkwVec {
        WkwVec::from_scalar(s)
    }
}

impl Add for WkwVec {
    type Output = Self;

    fn add(self, rhs: WkwVec) -> Self::Output {
        WkwVec {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

impl Div for WkwVec {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        WkwVec {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}

#[derive(Debug)]
struct MortonIdx(u64);

fn shuffle(v: u64) -> u64 {
    // take first 21 bits
    let mut z =       v & 0x00000000001fffff;
    z = (z | (z << 32)) & 0x001f00000000ffff;
    z = (z | (z << 16)) & 0x001f0000ff0000ff;
    z = (z | (z <<  8)) & 0x100f00f00f00f00f;
    z = (z | (z <<  4)) & 0x100f00f00f00f00f;
    z = (z | (z <<  2)) & 0x1249249249249249;

    z
}

fn unshuffle(z: u64) -> u64 {
    let mut v =       z & 0x1249249249249249;
    v = (v ^ (v >>  2)) & 0x100f00f00f00f00f;
    v = (v ^ (v >>  4)) & 0x100f00f00f00f00f;
    v = (v ^ (v >>  8)) & 0x001f0000ff0000ff;
    v = (v ^ (v >> 16)) & 0x001f00000000ffff;
    v = (v ^ (v >> 32)) & 0x00000000001fffff;

    v
}

impl<'a> From<&'a WkwVec> for MortonIdx {
    fn from(vec: &'a WkwVec) -> MortonIdx {
        MortonIdx(
            (shuffle(vec.x as u64) << 0) |
            (shuffle(vec.y as u64) << 1) |
            (shuffle(vec.z as u64) << 2)
        )
    }
}

impl From<MortonIdx> for WkwVec {
    fn from(idx: MortonIdx) -> WkwVec {
        WkwVec {
            x: unshuffle(idx.0 >> 0) as u32,
            y: unshuffle(idx.0 >> 1) as u32,
            z: unshuffle(idx.0 >> 2) as u32
        }
    }
}

impl From<MortonIdx> for u64 {
    fn from(idx: MortonIdx) -> u64 { idx.0 }
}

impl From<u64> for MortonIdx {
    fn from(idx: u64) -> MortonIdx { MortonIdx(idx) }
}

struct WkwMat<'a> {
    data: &'a mut [u8],
    shape: WkwVec,
    width: usize
}

impl<'a> WkwMat<'a> {
    fn new(data: &mut [u8], shape: WkwVec, width: usize) -> Result<WkwMat> {
        // make sure that slice is large enough
        let numel = shape.x as usize * shape.y as usize * shape.z as usize;
        let expected_len: usize = numel * width;

        if data.len() != expected_len {
            return Err("Length of slice does not match expected size")
        }

        Ok(WkwMat {
            data: data,
            shape: shape,
            width: width
        })
    }

    fn offset(&self, pos: &WkwVec) -> usize {
        pos.x as usize + self.shape.x as usize * (
        pos.y as usize + self.shape.y as usize * pos.z as usize) * self.width
    }

    fn copy_from(&mut self, src: &WkwMat, off: &WkwVec) -> Result<()> {
        if self.width != src.width {
            return Err("Source and destination matrices do not match in width");
        }

        let end = off.clone() + src.shape;
        if !self.shape.is_larger_equal_than(&end){
            return Err("Trying to write out of bounds");
        }

        let src_ptr = src.data.as_ptr();
        let dst_ptr = self.data.as_mut_ptr();
        let stripe_len = src.shape.x as usize * src.width;

        for cur_z in 0..src.shape.z {
            for cur_y in 0..src.shape.y {
                unsafe {
                    // TODO: optimize
                    let cur_pos = WkwVec { x: 0u32, y: cur_y, z: cur_z };
                    let src_ptr_cur = src_ptr.offset(src.offset(&cur_pos) as isize);

                    let dst_pos = off.clone() + cur_pos;
                    let dst_ptr_cur = dst_ptr.offset(self.offset(&dst_pos) as isize);

                    // copy data
                    ptr::copy_nonoverlapping(src_ptr_cur, dst_ptr_cur, stripe_len);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct WkwFile<'a> {
    file: &'a File,
    header: Header,
    block_idx: Option<u64>
}

impl<'a> WkwFile<'a> {
    fn from_file(file: &'a mut File) -> Result<WkwFile> {
        let header = wkw_read_header(file).unwrap();

        // create file and seek to first block
        let mut wkw_file = WkwFile { file: file, header: header, block_idx: None };
        wkw_file.seek_block(0 as u64)?;

        Ok(wkw_file)
    }

    fn read_mat(&mut self, mat: &mut WkwMat, off: &WkwVec) -> Result<usize> {
        if !off.is_valid_offset() {
            return Err("Offset is invalid");
        }

        if !mat.shape.is_power_of_two()
        || !mat.shape.is_larger_equal_than(off) {
            return Err("Shape of matrix is invalid");
        }

        let block_side_len = self.header.voxels_per_block_dim as u32;
        let block_ids = off.clone() / block_side_len.into();
        let block_idx = u64::from(MortonIdx::from(&block_ids));

        let blocks_per_dim = mat.shape.clone() / block_side_len.into();
        let block_count =
            blocks_per_dim.x as usize *
            blocks_per_dim.y as usize *
            blocks_per_dim.z as usize;

        // seek to start
        self.seek_block(block_idx)?;

        for cur_idx in 0..block_count {
            // read a block
            let mut buf = vec![0 as u8; self.header.block_size];
            self.read_block(buf.as_mut_slice())?;

            // build matrix arround buffer
            let buf_mat = WkwMat::new(
                buf.as_mut_slice(),
                WkwVec::from_scalar(self.header.voxels_per_block_dim as u32),
                self.header.voxel_size as usize).unwrap();

            // determine target position
            let cur_pos = WkwVec::from(MortonIdx::from(cur_idx as u64));

            // copy to target
            mat.copy_from(&buf_mat, &cur_pos)?;
        }

        Ok(mat.data.len())
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

fn main() {
    let wkw_path = "/home/amotta/Desktop/test.wkw";

    let mut file = File::open(wkw_path).unwrap();
    let mut wkw_file = WkwFile::from_file(&mut file).unwrap();

    println!("Header: {:#?}", wkw_file.header);

    // allocate buffer matrix
    let mut buf = vec![0 as u8; 128 * 128 * 128];

    let mut buf_mat = WkwMat::new(
        buf.as_mut_slice(),
        WkwVec::from_scalar(128 as u32),
        wkw_file.header.voxel_size as usize).unwrap();
    let pos = WkwVec::from_scalar(128);

    wkw_file.read_mat(&mut buf_mat, &pos).unwrap();

    println!("{:#?}", buf_mat.data);
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
