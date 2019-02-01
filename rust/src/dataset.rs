use crate::{BlockType, Box3, File, Header, Mat, Result, Vec3};
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug)]
pub struct Dataset {
    root: PathBuf,
    header: Header,
}

static HEADER_FILE_NAME: &'static str = "header.wkw";

impl Dataset {
    pub fn new(root: &Path) -> Result<Dataset> {
        // read required header file
        let header = Self::read_header_file(root)?;
        let root = root.to_owned();

        Ok(Dataset { root, header })
    }

    fn read_header_file(root: &Path) -> Result<Header> {
        let mut header_file_path = PathBuf::from(root);
        header_file_path.push(HEADER_FILE_NAME);

        let mut header_file = match fs::File::open(header_file_path) {
            Ok(header_file) => Ok(header_file),
            Err(_) => Err("Could not open dataset header file"),
        }?;

        Header::read(&mut header_file)
    }

    pub fn create(root: &Path, mut header: Header) -> Result<Dataset> {
        header.data_offset = 0;
        header.jump_table = None;

        fs::create_dir_all(root).or(Err("Could not create dataset directory"))?;
        Self::create_header_file(root, &header)?;

        let root = root.to_owned();
        Ok(Dataset { root, header })
    }

    fn create_header_file(root: &Path, header: &Header) -> Result<()> {
        let mut header_file_path = PathBuf::from(root);
        header_file_path.push(HEADER_FILE_NAME);

        let mut open_opts = fs::OpenOptions::new();
        open_opts.read(true).write(true).create_new(true);

        let mut file = match open_opts.open(header_file_path) {
            Ok(file) => Ok(file),
            Err(err) => match err.kind() {
                io::ErrorKind::AlreadyExists => Err("Dataset header file already exists"),
                _ => Err("Could not create dataset header file"),
            },
        }?;

        header.write(&mut file)
    }

    pub fn compress(&self, path: &Path) -> Result<Dataset> {
        let header = Header::compress(&self.header);
        Self::create(path, header)
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

                    // TODO(amotta): Distinguish between non-existen file and other failures (e.g.,
                    // due to missing file permissions). In the latter case we should return.
                    if let Ok(mut file) = File::open(&self.header, &cur_path) {
                        file.read_mat(cur_src_pos, mat, cur_dst_pos)?;
                    }
                }
            }
        }

        Ok(1 as usize)
    }

    pub fn write_mat(&self, dst_pos: Vec3, mat: &Mat) -> Result<usize> {
        // validate input matrix
        if mat.voxel_type != self.header.voxel_type {
            return Err("Input matrix has invalid voxel type");
        }

        if mat.voxel_size != self.header.voxel_size as usize {
            return Err("Input matrix has invalid voxel size");
        }

        let file_len_vx_log2 = self.header.file_len_vx_log2() as u32;
        if self.header.block_type == BlockType::LZ4 || self.header.block_type == BlockType::LZ4HC {
            let file_len_vec = Vec3::from(1 << file_len_vx_log2);
            let is_dst_aligned = dst_pos % file_len_vec == Vec3::from(0);
            let is_shape_aligned = mat.shape % file_len_vec == Vec3::from(0);
            if !is_dst_aligned || !is_shape_aligned {
                return Err("When writing compressed files, each file has to be \
                            written as a whole. Please pad your data so that all cubes \
                            are complete and the write position is block-aligned.");
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

                    let (_, mut file) = File::open_or_create(&self.header, &cur_path)?;
                    file.write_mat(cur_dst_pos, mat, cur_src_pos)?;
                }
            }
        }

        Ok(1 as usize)
    }
}
