use std::ptr;

use {Box3, Result, Vec3, VoxelType};

#[derive(Debug)]
pub struct Mat<'a> {
    data: &'a mut [u8],
    pub shape: Vec3,
    pub voxel_size: usize,
    pub voxel_type: VoxelType,
    pub data_in_c_order: bool,
}

impl<'a> Mat<'a> {
    pub fn new(
        data: &mut [u8],
        shape: Vec3,
        voxel_size: usize,
        voxel_type: VoxelType,
        data_in_c_order: bool,
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
            data_in_c_order: data_in_c_order,
        })
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
        let offset_vx = if self.data_in_c_order {
            pos.z + self.shape.z * (pos.y + self.shape.y * pos.x)
        } else {
            pos.x + self.shape.x * (pos.y + self.shape.y * pos.z)
        };
        offset_vx as usize * self.voxel_size
    }

    pub fn copy_as_fortran_order(&self, buffer: &mut Mat) -> Result<()> {
        if !self.data_in_c_order {
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

        fn linearize(x: usize, y: usize, z: usize, stride: &Vec<usize>) -> isize {
            (x * stride[0] + y * stride[1] + z * stride[2]) as isize
        }
        let src_ptr = self.data.as_ptr();
        let dst_ptr = buffer_data.as_mut_ptr();

        // Do continuous read in z. Last dim in Row-Major is continuous.
        for x in 0usize..x_length {
            for y in 0usize..y_length {
                for z in 0usize..z_length {
                    let row_major_index = linearize(x, y, z, &row_major_stride);
                    let column_major_index = linearize(x, y, z, &column_major_stride);
                    unsafe {
                        let cur_src_ptr = src_ptr.offset(row_major_index);
                        let cur_dst_ptr = dst_ptr.offset(column_major_index);
                        ptr::copy_nonoverlapping(cur_src_ptr, cur_dst_ptr, self.voxel_size);
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
        if self.data_in_c_order != src.data_in_c_order {
            return Err("Source and destination has to be the same order");
        }

        let length = src_box.width();
        // unified has fast to slow moving indices
        let unified_length = if self.data_in_c_order {
            length.flip()
        } else {
            length
        };
        let unified_dst_shape = if self.data_in_c_order {
            self.shape.flip()
        } else {
            self.shape
        };
        let unified_src_shape = if self.data_in_c_order {
            src.shape.flip()
        } else {
            src.shape
        };

        let stripe_len = src.voxel_size * unified_length.x as usize;

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

            for _ in 0..unified_length.z {
                let mut src_ptr_cur = src_ptr;
                let mut dst_ptr_cur = dst_ptr;

                for _ in 0..unified_length.y {
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
