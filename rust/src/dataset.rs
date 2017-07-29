extern crate walkdir;
use dataset::walkdir::{DirEntry, WalkDir, WalkDirIterator};

use ::{BlockType, File, Header, Result, Vec3, Box3, Mat};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug)]
pub struct Dataset<'a> {
    root: &'a Path,
    header: Header
}

static HEADER_FILE_NAME: &'static str = "header.wkw";

fn is_wkw_file(entry: &DirEntry) -> bool {
    let is_wkw = entry.file_name()
                      .to_str()
                      .map(|s| s.ends_with(".wkw"))
                      .unwrap_or(false);
    let is_file = entry.metadata()
                       .map(|m| m.is_file())
                       .unwrap_or(false);

    is_wkw && is_file
}

pub fn recover_header(root: &Path) -> Result<()> {
    // find an arbitrary .wkw file
    let mut walker = WalkDir::new(root)
                             .min_depth(3).max_depth(3)
                             .into_iter()
                             .filter_entry(|e| is_wkw_file(e));

    let wkw_file_entry = match walker.next() {
        Some(Ok(s)) => s,
        Some(Err(_)) => return Err("Error in directory walk"),
        None => return Err("No .wkw files found")
    };

    // open wkw file
    let file = File::open(wkw_file_entry.path())?;
    let mut header = file.header().clone();

    Dataset::create_header_file(root, &mut header)
}

impl<'a> Dataset<'a> {
    pub fn new(root: &'a Path) -> Result<Dataset<'a>> {
        if !root.is_dir() {
            return Err("Dataset root is not a directory")
        }

        // read required header file
        let header = Self::read_header(root)?;

        Ok(Dataset {
            root: root,
            header: header
        })
    }

    pub fn create(root: &'a Path, mut header: Header) -> Result<Dataset<'a>> {
        // create directory hierarchy
        fs::create_dir_all(root)
           .or(Err("Could not create dataset directory"))?;

        // create header file
        Self::create_header_file(root, &mut header)?;
        Self::new(root)
    }

    pub fn compress(&self, path: &'a Path) -> Result<Dataset<'a>> {
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
            return Err("Header file already exists");
        }

        // create header file
        let mut file = fs::File::create(header_path)
                                .or(Err("Could not create header file"))?;

        header.write(&mut file)
    }

    pub fn header(&'a self) -> &'a Header { &self.header }

    pub fn read_mat(&self, src_pos: Vec3, mat: &mut Mat) -> Result<usize> {
        let bbox = Box3::from(mat.shape) + src_pos;
        let file_len_vx_log2 = self.header.file_len_vx_log2() as u32;

        // find files to load
        let bbox_files = Box3::new(
            bbox.min() >> file_len_vx_log2,
          ((bbox.max() - 1) >> file_len_vx_log2) + 1
        )?;

        for cur_z in bbox_files.min().z..bbox_files.max().z {
            for cur_y in bbox_files.min().y..bbox_files.max().y {
                for cur_x in bbox_files.min().x..bbox_files.max().x {
                    // file path to wkw file
                    let mut cur_path = PathBuf::from(self.root);
                    cur_path.push(format!("z{}", cur_z));
                    cur_path.push(format!("y{}", cur_y));
                    cur_path.push(format!("x{}.wkw", cur_x));

                    // bounding box
                    let cur_file_ids = Vec3 { x: cur_x, y: cur_y, z: cur_z };
                    let cur_file_box = Box3::new(
                        cur_file_ids << file_len_vx_log2,
                       (cur_file_ids + 1) << file_len_vx_log2)?;
                    let cur_box = cur_file_box.intersect(bbox);

                    // offsets
                    let cur_src_pos = cur_box.min() - cur_file_box.min();
                    let cur_dst_pos = cur_box.min() - src_pos;

                    // try to open file
                    if let Ok(mut file) = File::open(&cur_path) {
                        file.read_mat(cur_src_pos, mat, cur_dst_pos)?;
                    }
                }
            }
        }

        Ok(1 as usize)
    }

    pub fn write_mat(&self, dst_pos: Vec3, mat: &Mat) -> Result<usize> {
            // validate block type
            match self.header.block_type {
                BlockType::LZ4 => Err("Cannot write LZ4 blocks"),
                BlockType::LZ4HC => Err("Cannot write LZ4HC blocks"),
                _ => Ok(())
            }?;

            // validate input matrix
            if mat.voxel_type != self.header.voxel_type {
                return Err("Input matrix has invalid voxel type");
            }

            if mat.voxel_size != self.header.voxel_size as usize {
                return Err("Input matrix has invalid voxel size");
            }

            let bbox = Box3::from(mat.shape) + dst_pos;
            let file_len_vx_log2 = self.header.file_len_vx_log2() as u32;

            // find files to load
            let bbox_files = Box3::new(
                bbox.min() >> file_len_vx_log2,
              ((bbox.max() - 1) >> file_len_vx_log2) + 1
            )?;

            for cur_z in bbox_files.min().z..bbox_files.max().z {
                for cur_y in bbox_files.min().y..bbox_files.max().y {
                    for cur_x in bbox_files.min().x..bbox_files.max().x {
                        // file path to wkw file
                        let mut cur_path = PathBuf::from(self.root);
                        cur_path.push(format!("z{}", cur_z));
                        cur_path.push(format!("y{}", cur_y));
                        cur_path.push(format!("x{}.wkw", cur_x));

                        // bounding box
                        let cur_file_ids = Vec3 { x: cur_x, y: cur_y, z: cur_z };
                        let cur_file_box = Box3::new(
                            cur_file_ids << file_len_vx_log2,
                           (cur_file_ids + 1) << file_len_vx_log2)?;
                        let cur_box = cur_file_box.intersect(bbox);

                        // offsets
                        let cur_src_pos = cur_box.min() - dst_pos;
                        let cur_dst_pos = cur_box.min() - cur_file_box.min();

                        let mut file = File::open_or_create(&cur_path, &self.header)?;
                        file.write_mat(cur_dst_pos, mat, cur_src_pos)?;
                    }
                }
            }

            Ok(1 as usize)
    }

    pub fn verify_headers(&self) -> Result<bool> {
        // find an arbitrary .wkw file
        let walker = WalkDir::new(self.root)
                             .min_depth(3).max_depth(3)
                             .into_iter()
                             .filter_entry(|e| is_wkw_file(e));

        // this header will be used as template
        let ref mut dataset_header = self.header.clone();

        for entry in walker {
            let entry = entry.unwrap();
            let entry_path = entry.path();

            let wkw_file = File::open(&entry_path).unwrap();
            let wkw_header = wkw_file.header();

            // we want to test for equality up to offset
            dataset_header.data_offset = wkw_header.data_offset;

            if wkw_header != dataset_header {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub(crate) fn read_header(root: &Path) -> Result<Header> {
        let mut header_path = PathBuf::from(root);
        header_path.push(HEADER_FILE_NAME);

        let mut header_file_opt = fs::File::open(header_path);
        let header_file = match header_file_opt.as_mut() {
            Ok(header_file) => header_file,
            Err(_) => return Err("Could not open header file")
        };

        Header::read(header_file)
    }
}
