use ::{Box3, Result, Vec3};

#[derive(PartialEq, Debug)]
pub struct Morton(u64);

fn shuffle(v: u64) -> u64 {
    // take first 21 bits
    let mut z =       v & 0x00000000001fffff;
    z = (z | (z << 32)) & 0x001f00000000ffff;
    z = (z | (z << 16)) & 0x001f0000ff0000ff;
    z = (z | (z <<  8)) & 0x100f00f00f00f00f;
    z = (z | (z <<  4)) & 0x100f00f00f00f00f;
    z = (z | (z <<  2)) & 0x1249249249249249;

    z
}

fn unshuffle(z: u64) -> u64 {
    let mut v =       z & 0x1249249249249249;
    v = (v ^ (v >>  2)) & 0x10c30c30c30c30c3;
    v = (v ^ (v >>  4)) & 0x100f00f00f00f00f;
    v = (v ^ (v >>  8)) & 0x001f0000ff0000ff;
    v = (v ^ (v >> 16)) & 0x001f00000000ffff;
    v = (v ^ (v >> 32)) & 0x00000000001fffff;

    v
}

impl<'a> From<&'a Vec3> for Morton {
    fn from(vec: &'a Vec3) -> Morton {
        Morton(
            (shuffle(vec.x as u64) << 0) |
            (shuffle(vec.y as u64) << 1) |
            (shuffle(vec.z as u64) << 2)
        )
    }
}

impl From<Morton> for Vec3 {
    fn from(idx: Morton) -> Vec3 {
        Vec3 {
            x: unshuffle(idx.0 >> 0) as u32,
            y: unshuffle(idx.0 >> 1) as u32,
            z: unshuffle(idx.0 >> 2) as u32
        }
    }
}

impl From<Morton> for u64 {
    fn from(idx: Morton) -> u64 { idx.0 }
}

impl From<u64> for Morton {
    fn from(idx: u64) -> Morton { Morton(idx) }
}

pub struct Iter {
    max_log: u32,
    cur_log: u32,
    cur_idx: u64,
    query: Box3
}

impl Iter {
    pub fn new(log2: u32, query: Box3) -> Result<Iter> {
        let query = Box3::new(query.min(), query.max() - 1)?;

        Ok(Iter {
            max_log: log2,
            cur_log: log2 - 1,
            cur_idx: 0,
            query: query
        })
    }
}

impl Iterator for Iter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        while self.cur_log < self.max_log {
            let off = Vec3::from(Morton(self.cur_idx >> 3)) << (self.cur_log + 1);
            let bbox = Box3::from(Vec3::from((1 << self.cur_log) - 1)) + off;

            for z_idx in (self.cur_idx & 0b111)..8 {
                let cur_off = Vec3::from(Morton(z_idx as u64));
                let cur_box = bbox + (cur_off << self.cur_log);
                let cur_idx = (self.cur_idx & !0b111) | (z_idx & 0b111);

                let cur_overlap =
                    self.query.max() >= cur_box.min()
                 && self.query.min() <= cur_box.max();

                if cur_overlap && self.cur_log > 0 {
                    // need to dive deeper
                    self.cur_log = self.cur_log - 1;
                    self.cur_idx = cur_idx << 3;
                    break;
                } else {
                    // need to visit next field
                    self.cur_idx = cur_idx + 1;

                    while self.cur_idx & 0b111 == 0 {
                        self.cur_idx = self.cur_idx >> 3;
                        self.cur_log = self.cur_log + 1;
                    }
                }

                if cur_overlap {
                    return Some(cur_idx);
                }
            }
        }

        None
    }
}

#[test]
fn test_encoding() {
    assert!(Morton::from(&Vec3 { x: 0, y: 0, z: 0 }) == Morton::from(0  as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 0, z: 0 }) == Morton::from(1  as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 1, z: 0 }) == Morton::from(2  as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 1, z: 0 }) == Morton::from(3  as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 0, z: 1 }) == Morton::from(4  as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 0, z: 1 }) == Morton::from(5  as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 1, z: 1 }) == Morton::from(6  as u64));
    assert!(Morton::from(&Vec3 { x: 1, y: 1, z: 1 }) == Morton::from(7  as u64));
    assert!(Morton::from(&Vec3 { x: 2, y: 0, z: 0 }) == Morton::from(8  as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 2, z: 0 }) == Morton::from(16 as u64));
    assert!(Morton::from(&Vec3 { x: 0, y: 0, z: 2 }) == Morton::from(32 as u64));
}

#[test]
fn test_decoding() {
    assert!(Vec3 { x: 0, y: 0, z: 0 } == Vec3::from(Morton::from(0  as u64)));
    assert!(Vec3 { x: 1, y: 0, z: 0 } == Vec3::from(Morton::from(1  as u64)));
    assert!(Vec3 { x: 0, y: 1, z: 0 } == Vec3::from(Morton::from(2  as u64)));
    assert!(Vec3 { x: 1, y: 1, z: 0 } == Vec3::from(Morton::from(3  as u64)));
    assert!(Vec3 { x: 0, y: 0, z: 1 } == Vec3::from(Morton::from(4  as u64)));
    assert!(Vec3 { x: 1, y: 0, z: 1 } == Vec3::from(Morton::from(5  as u64)));
    assert!(Vec3 { x: 0, y: 1, z: 1 } == Vec3::from(Morton::from(6  as u64)));
    assert!(Vec3 { x: 1, y: 1, z: 1 } == Vec3::from(Morton::from(7  as u64)));
    assert!(Vec3 { x: 2, y: 0, z: 0 } == Vec3::from(Morton::from(8  as u64)));
    assert!(Vec3 { x: 0, y: 2, z: 0 } == Vec3::from(Morton::from(16 as u64)));
    assert!(Vec3 { x: 0, y: 0, z: 2 } == Vec3::from(Morton::from(32 as u64)));
}
