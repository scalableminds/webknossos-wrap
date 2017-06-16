use std::ptr;

use ::{Box3, Result, Vec3};

#[derive(Debug)]
pub struct Mat<'a> {
    data: &'a mut [u8],
    shape: Vec3,
    width: usize
}

impl<'a> Mat<'a> {
    pub fn new(data: &mut [u8], shape: Vec3, width: usize) -> Result<Mat> {
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
    pub fn shape(&self) -> Vec3 { self.shape }
    pub fn width(&self) -> usize { self.width }

    fn offset(&self, pos: Vec3) -> usize {
        pos.x as usize + self.shape.x as usize * (
        pos.y as usize + self.shape.y as usize * pos.z as usize) * self.width
    }

    pub fn copy_all_from(&mut self, dst_pos: Vec3, src: &Mat) -> Result<()> {
        let src_box = Box3::new(Vec3::from(0u32), src.shape)?;
        self.copy_from(dst_pos, src, src_box)
    }

    pub fn copy_from(&mut self, dst_pos: Vec3, src: &Mat, src_box: Box3) -> Result<()> {
        if self.width != src.width {
            return Err("Source and destination matrices do not match in width");
        }

        if src_box.max() > src.shape { return Err("Reading out of bounds"); }
        if dst_pos + src_box.width() > self.shape() { return Err("Writing out of bounds"); }

        let src_ptr = unsafe { src.data.as_ptr().offset(src.offset(src_box.min()) as isize) };
        let dst_ptr = unsafe { self.data.as_mut_ptr().offset(self.offset(dst_pos) as isize) };

        let len = src_box.width();
        let stripe_len = src.width * len.x as usize;

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
