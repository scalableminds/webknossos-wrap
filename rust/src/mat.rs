use std::ptr;

use vec::Vec;
use result::Result;

#[derive(Debug)]
pub struct Mat<'a> {
    data: &'a mut [u8],
    shape: Vec,
    width: usize
}

impl<'a> Mat<'a> {
    pub fn new(data: &mut [u8], shape: Vec, width: usize) -> Result<Mat> {
        // make sure that slice is large enough
        let numel = shape.x as usize * shape.y as usize * shape.z as usize;
        let expected_len: usize = numel * width;

        if data.len() != expected_len {
            return Err("Length of slice does not match expected size")
        }

        Ok(Mat {
            data: data,
            shape: shape,
            width: width
        })
    }

    pub fn as_slice(&'a self) -> &'a [u8] { self.data }
    pub fn as_mut_slice(&'a mut self) -> &'a mut [u8] { self.data }
    pub fn shape(&self) -> &Vec { &self.shape }
    pub fn width(&self) -> usize { self.width }

    fn offset(&self, pos: &Vec) -> usize {
        pos.x as usize + self.shape.x as usize * (
        pos.y as usize + self.shape.y as usize * pos.z as usize) * self.width
    }

    pub fn copy_from(&mut self, src: &Mat, off: &Vec) -> Result<()> {
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
                    let cur_pos = Vec { x: 0u32, y: cur_y, z: cur_z };
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
