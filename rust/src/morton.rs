use std::cmp;
use {Box3, Result, Vec3};

#[derive(PartialEq, Debug)]
pub struct Morton(u64);

fn shuffle(v: u64) -> u64 {
    // take first 21 bits
    let mut z = v & 0x00000000001fffff;
    z = (z | (z << 32)) & 0x001f00000000ffff;
    z = (z | (z << 16)) & 0x001f0000ff0000ff;
    z = (z | (z << 8)) & 0x100f00f00f00f00f;
    z = (z | (z << 4)) & 0x100f00f00f00f00f;
    z = (z | (z << 2)) & 0x1249249249249249;

    z
}

fn unshuffle(z: u64) -> u64 {
    let mut v = z & 0x1249249249249249;
    v = (v ^ (v >> 2)) & 0x10c30c30c30c30c3;
    v = (v ^ (v >> 4)) & 0x100f00f00f00f00f;
    v = (v ^ (v >> 8)) & 0x001f0000ff0000ff;
    v = (v ^ (v >> 16)) & 0x001f00000000ffff;
    v = (v ^ (v >> 32)) & 0x00000000001fffff;

    v
}

impl<'a> From<&'a Vec3> for Morton {
    fn from(vec: &'a Vec3) -> Morton {
        Morton(
            (shuffle(vec.x as u64) << 0)
                | (shuffle(vec.y as u64) << 1)
                | (shuffle(vec.z as u64) << 2),
        )
    }
}

impl From<Morton> for Vec3 {
    fn from(idx: Morton) -> Vec3 {
        Vec3 {
            x: unshuffle(idx.0 >> 0) as u32,
            y: unshuffle(idx.0 >> 1) as u32,
            z: unshuffle(idx.0 >> 2) as u32,
        }
    }
}

impl From<Morton> for u64 {
    fn from(idx: Morton) -> u64 {
        idx.0
    }
}

impl From<u64> for Morton {
    fn from(idx: u64) -> Morton {
        Morton(idx)
    }
}

pub struct Iter {
    idx: u64,
    end: u64,
    log2: u32,
    bbox: Box3,
}

impl Iter {
    pub fn new(log2: u32, bbox: Box3) -> Result<Iter> {
        Ok(Iter {
            idx: 0,
            end: 0,
            log2: log2,
            bbox: bbox,
        })
    }

    fn find_range(&self) -> Option<(u64, u64)> {
        let max_idx = 1 << (3 * self.log2 as u64);

        // initialize state
        let mut cur_idx = self.idx;
        let mut cur_log2 = cmp::min(self.log2, cur_idx.trailing_zeros() / 3);

        while cur_idx < max_idx {
            let numel = 1 << (3 * cur_log2);
            let off = Vec3::from(Morton(cur_idx));

            let bbox = Box3::from(Vec3::from(1 << cur_log2)) + off;
            let bbox_inter = bbox.intersect(self.bbox);

            if bbox == bbox_inter {
                // return range
                let end_idx = cur_idx + numel;
                return Some((cur_idx, end_idx));
            } else if !bbox_inter.is_empty() {
                // need to refine
                debug_assert!(cur_log2 > 0);
                cur_log2 = cur_log2 - 1;
            } else {
                // skip cube
                cur_idx = cur_idx + numel;
                cur_log2 = cur_idx.trailing_zeros() / 3;
            }
        }

        None
    }
}

impl Iterator for Iter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.end == self.idx {
            // find next range
            match self.find_range() {
                Some((start, end)) => {
                    self.idx = start;
                    self.end = end;
                }
                None => return None,
            }
        }

        // advance
        self.idx = self.idx + 1;
        Some(self.idx as u64 - 1)
    }
}

#[test]
fn test_encoding() {
    assert!(Morton::from(&Vec3 { x: 0, y: 0, z: 0 }) == Morton::from(0 as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 0, z: 0 }) == Morton::from(1 as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 1, z: 0 }) == Morton::from(2 as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 1, z: 0 }) == Morton::from(3 as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 0, z: 1 }) == Morton::from(4 as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 0, z: 1 }) == Morton::from(5 as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 1, z: 1 }) == Morton::from(6 as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 1, z: 1 }) == Morton::from(7 as u64));
    assert!(Morton::from(&Vec3 { x: 2, y: 0, z: 0 }) == Morton::from(8 as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 2, z: 0 }) == Morton::from(16 as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 0, z: 2 }) == Morton::from(32 as u64));
}

#[test]
fn test_decoding() {
    assert!(Vec3 { x: 0, y: 0, z: 0 } == Vec3::from(Morton::from(0 as u64)));
    assert!(Vec3 { x: 1, y: 0, z: 0 } == Vec3::from(Morton::from(1 as u64)));
    assert!(Vec3 { x: 0, y: 1, z: 0 } == Vec3::from(Morton::from(2 as u64)));
    assert!(Vec3 { x: 1, y: 1, z: 0 } == Vec3::from(Morton::from(3 as u64)));
    assert!(Vec3 { x: 0, y: 0, z: 1 } == Vec3::from(Morton::from(4 as u64)));
    assert!(Vec3 { x: 1, y: 0, z: 1 } == Vec3::from(Morton::from(5 as u64)));
    assert!(Vec3 { x: 0, y: 1, z: 1 } == Vec3::from(Morton::from(6 as u64)));
    assert!(Vec3 { x: 1, y: 1, z: 1 } == Vec3::from(Morton::from(7 as u64)));
    assert!(Vec3 { x: 2, y: 0, z: 0 } == Vec3::from(Morton::from(8 as u64)));
    assert!(Vec3 { x: 0, y: 2, z: 0 } == Vec3::from(Morton::from(16 as u64)));
    assert!(Vec3 { x: 0, y: 0, z: 2 } == Vec3::from(Morton::from(32 as u64)));
}
