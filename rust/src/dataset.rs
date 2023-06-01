use std::fs;
use std::path::{Path, PathBuf};
use {BlockType, Box3, File, Header, Mat, Result, Vec3};

#[derive(Debug, Clone)]
pub struct Dataset {
    root: PathBuf,
    header: Header,
}

static HEADER_FILE_NAME: &str = "header.wkw";

impl Dataset {
    pub fn new(root: &Path) -> Result<Dataset> {
        if !root.is_dir() {
            return Err(format!("Dataset root {:?} is not a directory", &root));
        }

        // read required header file
        let header = Self::read_header(root)?;

        Ok(Dataset {
            root: root.to_owned(),
            header,
        })
    }

    pub fn create(root: &Path, mut header: Header) -> Result<Dataset> {
        // create directory hierarchy
        fs::create_dir_all(root).or(Err(format!(
            "Could not create dataset directory {:?}",
            &root
        )))?;

        // create header file
        Self::create_header_file(root, &mut header)?;
        Self::new(root)
    }

    pub fn compress(&self, path: &Path) -> Result<Dataset> {
        let header = Header::compress(&self.header);
        Self::create(path, header)
    }

    fn create_header_file(root: &Path, header: &mut Header) -> Result<()> {
        header.data_offset = 0;
        header.jump_table = None;

        // build path to header file
        let mut header_path = PathBuf::from(root);
        header_path.push(HEADER_FILE_NAME);

        if header_path.exists() {
            return Err(format!("Header {:?} file already exists", &header_path));
        }

        // create header file
        let mut file = fs::File::create(&header_path).or_else(|err| {
            Err(format!(
                "Could not create header file {:?}: {}",
                &header_path, err
            ))
        })?;

        header.write(&mut file)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn read_mat(&self, src_pos: Vec3, mat: &mut Mat) -> Result<usize> {
        let bbox = Box3::from(mat.shape) + src_pos;
        let file_len_vx_log2 = self.header.file_len_vx_log2() as u32;

        // find files to load
        let bbox_files = Box3::new(
            bbox.min() >> file_len_vx_log2,
            ((bbox.max() - 1) >> file_len_vx_log2) + 1,
        )?;

        for cur_z in bbox_files.min().z..bbox_files.max().z {
            for cur_y in bbox_files.min().y..bbox_files.max().y {
                for cur_x in bbox_files.min().x..bbox_files.max().x {
                    // file path to wkw file
                    let mut cur_path = self.root.clone();
                    cur_path.push(format!("z{}", cur_z));
                    cur_path.push(format!("y{}", cur_y));
                    cur_path.push(format!("x{}.wkw", cur_x));

                    // bounding box
                    let cur_file_ids = Vec3 {
                        x: cur_x,
                        y: cur_y,
                        z: cur_z,
                    };
                    let cur_file_box = Box3::new(
                        cur_file_ids << file_len_vx_log2,
                        (cur_file_ids + 1) << file_len_vx_log2,
                    )?;
                    let cur_box = cur_file_box.intersect(bbox);

                    // offsets
                    let cur_src_pos = cur_box.min() - cur_file_box.min();
                    let cur_dst_pos = cur_box.min() - src_pos;

                    // try to open file
                    if let Ok(mut file) = File::open(&cur_path) {
                        match file.read_mat(cur_src_pos, mat, cur_dst_pos) {
                            Ok(_) => {}
                            Err(err) => {
                                return Err(format!(
                                    "Error while reading from file {:?}: {}",
                                    &cur_path, err
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(1 as usize)
    }

    pub fn write_mat(&self, dst_pos: Vec3, mat: &Mat) -> Result<usize> {
        // validate input matrix
        if mat.voxel_type != self.header.voxel_type {
            return Err(format!(
                "Input matrix has invalid voxel type {:?} != {:?}",
                mat.voxel_type, self.header.voxel_type
            ));
        }

        if mat.voxel_size != self.header.voxel_size as usize {
            return Err(format!(
                "Input matrix has invalid voxel size {} != {}",
                mat.voxel_size, self.header.voxel_size as usize
            ));
        }

        let file_len_vx_log2 = self.header.file_len_vx_log2() as u32;
        if self.header.block_type == BlockType::LZ4 || self.header.block_type == BlockType::LZ4HC {
            let file_len_vec = Vec3::from(1 << file_len_vx_log2);
            let is_dst_aligned = dst_pos % file_len_vec == Vec3::from(0);
            let is_shape_aligned = mat.shape % file_len_vec == Vec3::from(0);
            if !is_dst_aligned || !is_shape_aligned {
                return Err(String::from(
                    "When writing compressed files, each file has to be \
                            written as a whole. Please pad your data so that all cubes \
                            are complete and the write position is block-aligned.",
                ));
            }
        };

        let bbox = Box3::from(mat.shape) + dst_pos;

        // find files to load
        let bbox_files = Box3::new(
            bbox.min() >> file_len_vx_log2,
            ((bbox.max() - 1) >> file_len_vx_log2) + 1,
        )?;

        for cur_z in bbox_files.min().z..bbox_files.max().z {
            for cur_y in bbox_files.min().y..bbox_files.max().y {
                for cur_x in bbox_files.min().x..bbox_files.max().x {
                    // file path to wkw file
                    let mut cur_path = self.root.clone();
                    cur_path.push(format!("z{}", cur_z));
                    cur_path.push(format!("y{}", cur_y));
                    cur_path.push(format!("x{}.wkw", cur_x));

                    // bounding box
                    let cur_file_ids = Vec3 {
                        x: cur_x,
                        y: cur_y,
                        z: cur_z,
                    };
                    let cur_file_box = Box3::new(
                        cur_file_ids << file_len_vx_log2,
                        (cur_file_ids + 1) << file_len_vx_log2,
                    )?;
                    let cur_box = cur_file_box.intersect(bbox);

                    // offsets
                    let cur_src_pos = cur_box.min() - dst_pos;
                    let cur_dst_pos = cur_box.min() - cur_file_box.min();

                    let mut file = match File::open_or_create(&cur_path, &self.header) {
                        Ok(file) => file,
                        Err(err) => {
                            return Err(format!(
                                "Error while open file {:?} for writing: {}",
                                &cur_path, err
                            ));
                        }
                    };
                    match file.write_mat(cur_dst_pos, mat, cur_src_pos) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(format!(
                                "Error while writing to file {:?}: {}",
                                &cur_path, err
                            ));
                        }
                    }
                }
            }
        }

        Ok(1 as usize)
    }

    pub(crate) fn read_header(root: &Path) -> Result<Header> {
        let mut header_path = PathBuf::from(root);
        header_path.push(HEADER_FILE_NAME);

        let mut header_file_opt = fs::File::open(&header_path);
        let header_file = match header_file_opt.as_mut() {
            Ok(header_file) => header_file,
            Err(_) => return Err(format!("Could not open header file {:?}", header_path)),
        };

        Header::read(header_file)
    }
}
