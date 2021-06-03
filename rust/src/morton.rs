use std::cmp;
use {Box3, Result, Vec3};

#[derive(PartialEq, Debug)]
pub struct Morton(u64);

impl<'a> From<&'a Vec3> for Morton {
    fn from(vec: &'a Vec3) -> Morton {
        let x = vec.x as u64;
        let y = vec.y as u64;
        let z = vec.z as u64;
        let mut idx = 0u64;
        let bit_length = 64 - (std::cmp::max(x, std::cmp::max(y, z)) + 1).leading_zeros();
        for i in 0..bit_length {
            idx |= ((x & (1 << i)) << (2 * i))
                | ((y & (1 << i)) << (2 * i + 1))
                | ((z & (1 << i)) << (2 * i + 2))
        }
        Morton(idx)
    }
}

impl From<Morton> for Vec3 {
    fn from(idx: Morton) -> Vec3 {
        let mut idx = idx.0;
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;
        let mut bit = 0;

        while idx > 0 {
            x |= (idx & 1) << bit;
            idx >>= 1;
            y |= (idx & 1) << bit;
            idx >>= 1;
            z |= (idx & 1) << bit;
            idx >>= 1;
            bit += 1;
        }
        Vec3 {
            x: x as u32,
            y: y as u32,
            z: z as u32,
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
            log2,
            bbox,
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
                cur_log2 -= 1;
            } else {
                // skip cube
                cur_idx += numel;
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
        self.idx += 1;
        Some(self.idx as u64 - 1)
    }
}

#[test]
fn test_encoding() {
    assert_eq!(
        Morton::from(&Vec3 { x: 0, y: 0, z: 0 }),
        Morton::from(0 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 1, y: 0, z: 0 }),
        Morton::from(1 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 0, y: 1, z: 0 }),
        Morton::from(2 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 1, y: 1, z: 0 }),
        Morton::from(3 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 0, y: 0, z: 1 }),
        Morton::from(4 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 1, y: 0, z: 1 }),
        Morton::from(5 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 0, y: 1, z: 1 }),
        Morton::from(6 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 1, y: 1, z: 1 }),
        Morton::from(7 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 2, y: 0, z: 0 }),
        Morton::from(8 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 0, y: 2, z: 0 }),
        Morton::from(16 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 0, y: 0, z: 2 }),
        Morton::from(32 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 29, y: 20, z: 3 }),
        Morton::from(13029 as u64)
    );
    assert_eq!(
        Morton::from(&Vec3 { x: 23, y: 20, z: 3 }),
        Morton::from(12525 as u64)
    );
}

#[test]
fn test_decoding() {
    assert_eq!(
        Vec3 { x: 0, y: 0, z: 0 },
        Vec3::from(Morton::from(0 as u64))
    );
    assert_eq!(
        Vec3 { x: 1, y: 0, z: 0 },
        Vec3::from(Morton::from(1 as u64))
    );
    assert_eq!(
        Vec3 { x: 0, y: 1, z: 0 },
        Vec3::from(Morton::from(2 as u64))
    );
    assert_eq!(
        Vec3 { x: 1, y: 1, z: 0 },
        Vec3::from(Morton::from(3 as u64))
    );
    assert_eq!(
        Vec3 { x: 0, y: 0, z: 1 },
        Vec3::from(Morton::from(4 as u64))
    );
    assert_eq!(
        Vec3 { x: 1, y: 0, z: 1 },
        Vec3::from(Morton::from(5 as u64))
    );
    assert_eq!(
        Vec3 { x: 0, y: 1, z: 1 },
        Vec3::from(Morton::from(6 as u64))
    );
    assert_eq!(
        Vec3 { x: 1, y: 1, z: 1 },
        Vec3::from(Morton::from(7 as u64))
    );
    assert_eq!(
        Vec3 { x: 2, y: 0, z: 0 },
        Vec3::from(Morton::from(8 as u64))
    );
    assert_eq!(
        Vec3 { x: 0, y: 2, z: 0 },
        Vec3::from(Morton::from(16 as u64))
    );
    assert_eq!(
        Vec3 { x: 0, y: 0, z: 2 },
        Vec3::from(Morton::from(32 as u64))
    );
    assert_eq!(
        Vec3 { x: 29, y: 20, z: 3 },
        Vec3::from(Morton::from(13029 as u64))
    );
    assert_eq!(
        Vec3 { x: 23, y: 20, z: 3 },
        Vec3::from(Morton::from(12525 as u64))
    );
}
