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
            return Err(format!("Length of slice does not match expected size {} != {}", data.len(), expected_len));
        }

        if voxel_size % voxel_type.size() != 0 {
            return Err(format!("Voxel size must be a multiple of voxel type size {} % {} != 0", voxel_size, voxel_type.size()));
        }

        Ok(Mat {
            data,
            shape,
            voxel_size,
            voxel_type,
            data_in_c_order,
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
        // Early usize cast is necessary as overflows happen
        let offset_vx = if self.data_in_c_order {
            pos.z as usize
                + self.shape.z as usize * (pos.y as usize + self.shape.y as usize * pos.x as usize)
        } else {
            pos.x as usize
                + self.shape.x as usize * (pos.y as usize + self.shape.y as usize * pos.z as usize)
        };
        offset_vx * self.voxel_size
    }

    pub fn copy_as_fortran_order(&self, buffer: &mut Mat, src_bbox: Box3) -> Result<()> {
        if !self.data_in_c_order {
            return Err(String::from("Mat is already in fortran order"));
        }
        if self.voxel_size != buffer.voxel_size {
            return Err(format!("Matrices mismatch in voxel size {} != {}", self.voxel_size, buffer.voxel_size));
        }
        if self.voxel_type != buffer.voxel_type {
            return Err(format!("Matrices mismatch in voxel type {:?} != {:?}", self.voxel_type, buffer.voxel_type));
        }
        if self.shape != buffer.shape {
            return Err(format!("Matrices mismatch in shape {:?} != {:?}", self.shape, buffer.shape));
        }

        let buffer_data = buffer.as_mut_slice();

        let x_length = self.shape.x as usize;
        let y_length = self.shape.y as usize;
        let num_channel = self.voxel_size / self.voxel_type.size();
        let item_size = self.voxel_size / num_channel;
        //println!("num_channel {}", num_channel);

        //println!("self.shape {:?}", self.shape);

        let row_major_stride: Vec<usize> = vec![
            item_size,
            y_length * x_length * self.voxel_size,
            y_length * self.voxel_size,
            self.voxel_size,
        ];

        let column_major_stride: Vec<usize> = vec![
            item_size,
            self.voxel_size,
            x_length * self.voxel_size,
            x_length * y_length * self.voxel_size,
        ];

        //println!("row_major_stride {:?}", row_major_stride);
        //println!("column_major_stride {:?}", column_major_stride);

        fn linearize(channel: usize, x: usize, y: usize, z: usize, stride: &[usize]) -> isize {
            (channel * stride[0] + x * stride[1] + y * stride[2] + z * stride[3]) as isize
        }

        let src_ptr = self.data.as_ptr();
        let dst_ptr = buffer_data.as_mut_ptr();

        let from = src_bbox.min();
        let to = src_bbox.max();

        // Do continuous read in z. Last dim in Row-Major is continuous.
        let stripe_len = item_size * num_channel;
        for x in from.x as usize..to.x as usize {
            for y in from.y as usize..to.y as usize {
                for z in from.z as usize..to.z as usize {
                    let row_major_index = linearize(0, x, y, z, &row_major_stride);
                    let column_major_index = linearize(0, x, y, z, &column_major_stride);
                    unsafe {
                        //println!("x {}, y {}, z {}, channel {}, column_major_index {}, row_major_index {}", x, y, z, 0, column_major_index, row_major_index);
                        //println!("cur_src_ptr: {}", *src_ptr.offset(row_major_index));
                        //println!("cur_dst_ptr: {}", *dst_ptr.offset(column_major_index));
                        ptr::copy_nonoverlapping(src_ptr.offset(row_major_index), dst_ptr.offset(column_major_index), stripe_len);
                    }

                }
            }
        }
        Ok(())
    }

    pub fn copy_from_order_agnostic(
        &mut self,
        dst_pos: Vec3,
        src: &Mat,
        src_box: Box3,
        intermediate_buffer: &mut Mat,
    ) -> Result<()> {
        if self.data_in_c_order {
            return Err(String::from("copy_from_order_agnostic has to be called on a fortran order buffer."));
        }

        if src.data_in_c_order {
            let num_channel = self.voxel_size / self.voxel_type.size();
            if num_channel == 1 {
                intermediate_buffer.copy_from(dst_pos, src, src_box)?;
            } else {
                intermediate_buffer.copy_from_and_put_channels_last(dst_pos, src, src_box)?;
            }
            let dst_bbox = Box3::new(dst_pos, dst_pos + src_box.width())?;
            intermediate_buffer.copy_as_fortran_order(self, dst_bbox)
        } else {
            self.copy_from(dst_pos, src, src_box)
        }
    }

    pub fn copy_from(&mut self, dst_pos: Vec3, src: &Mat, src_box: Box3) -> Result<()> {
        // make sure that matrices are matching
        if self.voxel_size != src.voxel_size {
            return Err(format!("Matrices mismatch in voxel size {} != {}", self.voxel_size, src.voxel_size));
        }
        if self.voxel_type != src.voxel_type {
            return Err(format!("Matrices mismatch in voxel type {:?} != {:?}", self.voxel_type, src.voxel_type));
        }
        if !(src_box.max() < (src.shape + 1)) {
            return Err(String::from("Reading out of bounds"));
        }
        if !(dst_pos + src_box.width() < (self.shape + 1)) {
            return Err(String::from("Writing out of bounds"));
        }
        if self.data_in_c_order != src.data_in_c_order {
            return Err(String::from("Source and destination has to be the same order"));
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
            //println!("src.offset(src_box.min()): {:?}", src_box.min());
            //println!("src.offset(src_box.min()): {}", src.offset(src_box.min()));
            let mut src_ptr = src.data.as_ptr().add(src.offset(src_box.min()));
            let mut dst_ptr = self.data.as_mut_ptr().add(self.offset(dst_pos));

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

    pub fn copy_from_and_put_channels_last(&mut self, dst_pos: Vec3, src: &Mat, src_box: Box3) -> Result<()> {
        // make sure that matrices are matching
        if self.voxel_size != src.voxel_size {
            return Err(format!("Matrices mismatch in voxel size {} != {}", self.voxel_size, src.voxel_size));
        }
        if self.voxel_type != src.voxel_type {
            return Err(format!("Matrices mismatch in voxel type {:?} != {:?}", self.voxel_type, src.voxel_type));
        }
        if !(src_box.max() < (src.shape + 1)) {
            return Err(String::from("Reading out of bounds"));
        }
        if !(dst_pos + src_box.width() < (self.shape + 1)) {
            return Err(String::from("Writing out of bounds"));
        }
        if self.data_in_c_order != src.data_in_c_order {
            return Err(String::from("Source and destination has to be the same order"));
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

        //println!("length {:?}", length);
        //println!("unified_length {:?}", unified_length);
        //println!("dst_shape {:?}", self.shape);
        //println!("unified_dst_shape {:?}", unified_dst_shape);
        //println!("src_shape {:?}", src.shape);
        //println!("unified_src_shape {:?}", unified_src_shape);

        let num_channel = self.voxel_size / self.voxel_type.size();
        let item_size = self.voxel_size / num_channel;

        let channel_last_stride: Vec<usize> = vec![
            (unified_src_shape.x * unified_src_shape.y * unified_src_shape.z) as usize * item_size,
            (unified_src_shape.x * unified_src_shape.y) as usize * item_size,
            unified_src_shape.x as usize * item_size,
            item_size,
        ];

        let channel_first_stride: Vec<usize> = vec![
            item_size,
            (unified_dst_shape.x * unified_dst_shape.y) as usize * self.voxel_size,
            unified_dst_shape.x as usize as usize * self.voxel_size,
            self.voxel_size,
        ];

        fn linearize(channel: usize, x: usize, y: usize, z: usize, stride: &[usize]) -> isize {
            (channel * stride[0] + x * stride[1] + y * stride[2] + z * stride[3]) as isize
        }

        unsafe {
            let src_ptr = src.data.as_ptr().add(src.offset(src_box.min()) / num_channel);
            let dst_ptr = self.data.as_mut_ptr().add(self.offset(dst_pos));

            //println!("scr_box.min() {:?}", src_box.min());
            //println!("src_ptr offset {}", src.offset(src_box.min()));
            //println!("dst_pos {:?}", dst_pos);
            //println!("dst_pos offset {}", self.offset(dst_pos));

            for channel in 0..num_channel {
                for x in 0..unified_length.z {
                    for y in 0..unified_length.y {
                        for z in 0..unified_length.x {
                            let channel_last_index = linearize(channel, x as usize, y as usize, z as usize, &channel_last_stride);
                            let channel_first_index = linearize(channel, x as usize, y as usize, z as usize, &channel_first_stride);
                            //println!("x: {}, y: {}, z: {}, channel: {}, channel_last_index: {}, channel_first_index: {}", x, y, z, channel, channel_last_index, channel_first_index);
                            //println!("{:?}", *src_ptr.offset(channel_last_index));
                            ptr::copy_nonoverlapping(src_ptr.offset(channel_last_index), dst_ptr.offset(channel_first_index), item_size);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
