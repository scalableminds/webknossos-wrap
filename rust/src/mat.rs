use std::ptr;

use {Box3, Result, Vec3, VoxelType};

#[derive(Debug)]
pub struct Mat<'a> {
    data: &'a mut [u8],
    pub shape: Vec3,
    pub voxel_size: usize,
    pub voxel_type: VoxelType,
    is_fortran_order: bool,
}

impl<'a> Mat<'a> {
    pub fn new(
        data: &mut [u8],
        shape: Vec3,
        voxel_size: usize,
        voxel_type: VoxelType,
        is_fortran_array: bool,
    ) -> Result<Mat> {
        // make sure that slice is large enough
        let numel = shape.x as usize * shape.y as usize * shape.z as usize;
        let expected_len = numel * voxel_size;
        if data.len() != expected_len {
            return Err("Length of slice does not match expected size");
        }

        if voxel_size % voxel_type.size() != 0 {
            return Err("Voxel size must be a multiple of voxel type size");
        }

        Ok(Mat {
            data: data,
            shape: shape,
            voxel_size: voxel_size,
            voxel_type: voxel_type,
            is_fortran_order: is_fortran_array,
        })
    }

    pub fn get_is_fortran_order(&self) -> bool {
        return self.is_fortran_order;
    }
    pub fn as_slice(&self) -> &[u8] {
        self.data
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    fn offset(&self, pos: Vec3) -> usize {
        if self.is_fortran_order {
            let offset_vx = pos.x as usize
                + self.shape.x as usize * (pos.y as usize + self.shape.y as usize * pos.z as usize);
            offset_vx * self.voxel_size
        } else {
            let offset_vx = pos.z as usize
                + self.shape.z as usize * (pos.y as usize + self.shape.y as usize * pos.x as usize);
            offset_vx * self.voxel_size
        }
    }

    pub fn write_fortran_order_to_buffer(&self, buffer: &mut Mat) -> Result<()> {
        if self.is_fortran_order {
            return Err("Mat is already in fortran order");
        }
        if self.voxel_size != buffer.voxel_size {
            return Err("Matrices mismatch in voxel size");
        }
        if self.voxel_type != buffer.voxel_type {
            return Err("Matrices mismatch in voxel type");
        }
        if self.shape != buffer.shape {
            return Err("Matrices mismatch in shape");
        }

        let buffer_data = buffer.as_mut_slice();

        let x_length = self.shape.x as usize;
        let y_length = self.shape.y as usize;
        let z_length = self.shape.z as usize;

        let row_major_stride: Vec<usize> = vec![
            z_length * y_length * self.voxel_size,
            z_length * self.voxel_size,
            self.voxel_size,
        ];
        let column_major_stride: Vec<usize> = vec![
            self.voxel_size,
            x_length * self.voxel_size,
            x_length * y_length * self.voxel_size,
        ];
        // Do continuous read in z. Last dim in Row-Major is continuous.
        for x in 0usize..x_length {
            for y in 0usize..y_length {
                for z in 0usize..z_length {
                    let row_major_index =
                        x * row_major_stride[0] + y * row_major_stride[1] + z * row_major_stride[2];
                    let column_major_index = x * column_major_stride[0]
                        + y * column_major_stride[1]
                        + z * column_major_stride[2];
                    for byte_offset in 0..self.voxel_size {
                        buffer_data[column_major_index as usize + byte_offset as usize] =
                            self.data[row_major_index as usize + byte_offset as usize];
                    }
                }
            }
        }
        Ok(())
    }

    pub fn copy_from(&mut self, dst_pos: Vec3, src: &Mat, src_box: Box3) -> Result<()> {
        // make sure that matrices are matching
        if self.voxel_size != src.voxel_size {
            return Err("Matrices mismatch in voxel size");
        }
        if self.voxel_type != src.voxel_type {
            return Err("Matrices mismatch in voxel type");
        }
        if !(src_box.max() < (src.shape + 1)) {
            return Err("Reading out of bounds");
        }
        if !(dst_pos + src_box.width() < (self.shape + 1)) {
            return Err("Writing out of bounds");
        }
        if self.is_fortran_order != src.get_is_fortran_order() {
            return Err("source and destination has to be the same order");
        }

        let width = src_box.width();
        // unified has fast to slow moving indices
        let unified_width = match self.is_fortran_order {
            true => width,
            false => Vec3 {
                x: width.z,
                y: width.y,
                z: width.x,
            },
        };
        let unified_dst_shape = match self.is_fortran_order {
            true => self.shape,
            false => Vec3 {
                x: self.shape.z,
                y: self.shape.y,
                z: self.shape.x,
            },
        };
        let unified_src_shape = match self.is_fortran_order {
            true => src.shape,
            false => Vec3 {
                x: src.shape.z,
                y: src.shape.y,
                z: src.shape.x,
            },
        };

        let stripe_len = src.voxel_size * unified_width.x as usize;

        let src_inner_offset = (unified_src_shape.x as usize * self.voxel_size) as isize;
        let src_outer_offset = (unified_src_shape.x as usize
            * unified_src_shape.y as usize
            * self.voxel_size) as isize;

        let dst_inner_offset = (unified_dst_shape.x as usize * self.voxel_size) as isize;
        let dst_outer_offset = (unified_dst_shape.x as usize
            * unified_dst_shape.y as usize
            * self.voxel_size) as isize;

        unsafe {
            let mut src_ptr = src.data.as_ptr().offset(src.offset(src_box.min()) as isize);
            let mut dst_ptr = self.data.as_mut_ptr().offset(self.offset(dst_pos) as isize);

            for _ in 0..unified_width.z {
                let mut src_ptr_cur = src_ptr;
                let mut dst_ptr_cur = dst_ptr;

                for _ in 0..unified_width.y {
                    // copy data
                    ptr::copy_nonoverlapping(src_ptr_cur, dst_ptr_cur, stripe_len);

                    // advance
                    src_ptr_cur = src_ptr_cur.offset(src_inner_offset);
                    dst_ptr_cur = dst_ptr_cur.offset(dst_inner_offset);
                }

                src_ptr = src_ptr.offset(src_outer_offset);
                dst_ptr = dst_ptr.offset(dst_outer_offset);
            }
        }
        Ok(())
    }
}
