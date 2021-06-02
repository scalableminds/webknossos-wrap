use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use {Box3, Dataset, File, Header, Mat, Result, Vec3};

#[derive(Clone, Debug)]
pub struct CachedDataset {
  root: PathBuf,
  header: Header,
  cache: Arc<Mutex<HashMap<PathBuf, Arc<RwLock<File>>>>>,
}

impl CachedDataset {
  pub fn new(root: &Path) -> Result<Self> {
    if !root.is_dir() {
      return Err(format!("Dataset root {:?} is not a directory", &root));
    }

    // read required header file
    let header = Dataset::read_header(root)?;

    Ok(Self {
      root: root.to_owned(),
      header,
      cache: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  pub fn header(&self) -> &Header {
    &self.header
  }

  fn get_file(&self, path: &Path) -> Option<Arc<RwLock<File>>> {
    let mut cache = self.cache.lock().unwrap();
    let cached_file = cache.get(&PathBuf::from(path)).cloned();
    if let Some(cached_file) = cached_file {
      return Some(cached_file);
    }
    match File::open(&path) {
      Ok(file) => {
        let file = Arc::new(RwLock::new(file));
        cache.insert(PathBuf::from(path), file.clone());
        Some(file)
      }
      Err(_) => None,
    }
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
          if let Some(file) = self.get_file(&cur_path) {
            file
              .write()
              .unwrap()
              .read_mat(cur_src_pos, mat, cur_dst_pos)?;
          }
        }
      }
    }

    Ok(1 as usize)
  }
}
