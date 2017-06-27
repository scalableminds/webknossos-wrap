use std::ptr;

use ::{Box3, Result, Vec3};

#[derive(Debug)]
pub struct Mat<'a> {
    data: &'a mut [u8],
    shape: Vec3,
    voxel_size: usize
}

impl<'a> Mat<'a> {
    pub fn new(data: &mut [u8], shape: Vec3, voxel_size: usize) -> Result<Mat> {
        // make sure that slice is large enough
        let numel = shape.x as usize * shape.y as usize * shape.z as usize;
        let expected_len = numel * voxel_size;

        if data.len() != expected_len {
            return Err("Length of slice does not match expected size")
        }

        Ok(Mat {
            data: data,
            shape: shape,
            voxel_size: voxel_size
        })
    }

    pub fn as_slice(&self) -> &[u8] { self.data }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { self.data }
    pub fn as_mut_ptr(&mut self) -> *mut u8 { self.data.as_mut_ptr() }

    pub fn shape(&self) -> Vec3 { self.shape }
    pub fn voxel_size(&self) -> usize { self.voxel_size }

    fn offset(&self, pos: Vec3) -> usize {
        let offset_vx =
            pos.x as usize + self.shape.x as usize * (
            pos.y as usize + self.shape.y as usize * pos.z as usize);

        offset_vx * self.voxel_size
    }

    pub fn copy_from(&mut self, dst_pos: Vec3, src: &Mat, src_box: Box3) -> Result<()> {
        if self.voxel_size != src.voxel_size {
            return Err("Matrices mismatch in voxel size");
        }

        if src_box.max() > src.shape { return Err("Reading out of bounds"); }
        if dst_pos + src_box.width() > self.shape() { return Err("Writing out of bounds"); }

        let src_ptr = unsafe { src.data.as_ptr().offset(src.offset(src_box.min()) as isize) };
        let dst_ptr = unsafe { self.data.as_mut_ptr().offset(self.offset(dst_pos) as isize) };

        let len = src_box.width();
        let stripe_len = src.voxel_size * len.x as usize;

        for cur_z in 0..len.z {
            for cur_y in 0..len.y {
                unsafe {
                    // TODO: optimize
                    let cur_pos = Vec3 { x: 0u32, y: cur_y, z: cur_z };
                    let src_ptr_cur = src_ptr.offset(src.offset(cur_pos) as isize);
                    let dst_ptr_cur = dst_ptr.offset(self.offset(cur_pos) as isize);

                    // copy data
                    ptr::copy_nonoverlapping(src_ptr_cur, dst_ptr_cur, stripe_len);
                }
            }
        }

        Ok(())
    }
}
