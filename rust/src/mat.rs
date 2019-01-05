use std::ptr;

use crate::{Box3, Result, Vec3, VoxelType};

#[derive(Debug)]
pub struct Mat<'a> {
    data: &'a mut [u8],
    pub shape: Vec3,
    pub voxel_size: usize,
    pub voxel_type: VoxelType
}

impl<'a> Mat<'a> {
    pub fn new(
        data: &mut [u8],
        shape: Vec3,
        voxel_size: usize,
        voxel_type: VoxelType
    ) -> Result<Mat> {
        // make sure that slice is large enough
        let numel = shape.x as usize * shape.y as usize * shape.z as usize;
        let expected_len = numel * voxel_size;

        if data.len() != expected_len {
            return Err("Length of slice does not match expected size")
        }

        if voxel_size % voxel_type.size() != 0 {
            return Err("Voxel size must be a multiple of voxel type size")
        }

        Ok(Mat {
            data: data,
            shape: shape,
            voxel_size: voxel_size,
            voxel_type: voxel_type
        })
    }

    pub fn as_slice(&self) -> &[u8] { self.data }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { self.data }
    pub fn as_mut_ptr(&mut self) -> *mut u8 { self.data.as_mut_ptr() }

    fn offset(&self, pos: Vec3) -> usize {
        let offset_vx =
            pos.x as usize + self.shape.x as usize * (
            pos.y as usize + self.shape.y as usize * pos.z as usize);

        offset_vx * self.voxel_size
    }

    pub fn copy_from(&mut self, dst_pos: Vec3, src: &Mat, src_box: Box3) -> Result<()> {
        // make sure that matrices are matching
        if self.voxel_size != src.voxel_size { return Err("Matrices mismatch in voxel size"); }
        if self.voxel_type != src.voxel_type { return Err("Matrices mismatch in voxel type"); }

        if !(src_box.max() < (src.shape + 1)) { return Err("Reading out of bounds"); }
        if !(dst_pos + src_box.width() < (self.shape + 1)) { return Err("Writing out of bounds"); }

        let len = src_box.width();
        let stripe_len = src.voxel_size * len.x as usize;

        let src_off_y = (src.shape.x as usize * src.voxel_size) as isize;
        let src_off_z = (src.shape.x as usize * src.shape.y as usize * self.voxel_size) as isize;

        let dst_off_y = (self.shape.x as usize * self.voxel_size) as isize;
        let dst_off_z = (self.shape.x as usize * self.shape.y as usize * self.voxel_size) as isize;

        unsafe {
            let mut src_ptr = src.data.as_ptr().offset(src.offset(src_box.min()) as isize);
            let mut dst_ptr = self.data.as_mut_ptr().offset(self.offset(dst_pos) as isize);

            for _ in 0..len.z {
                let mut src_ptr_cur = src_ptr;
                let mut dst_ptr_cur = dst_ptr;

                for _ in 0..len.y {
                    // copy data
                    ptr::copy_nonoverlapping(src_ptr_cur, dst_ptr_cur, stripe_len);

                    // advance
                    src_ptr_cur = src_ptr_cur.offset(src_off_y);
                    dst_ptr_cur = dst_ptr_cur.offset(dst_off_y);
                }

                src_ptr = src_ptr.offset(src_off_z);
                dst_ptr = dst_ptr.offset(dst_off_z);
            }
        }

        Ok(())
    }
}
