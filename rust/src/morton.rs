use vec::Vec;

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

impl<'a> From<&'a Vec> for Morton {
    fn from(vec: &'a Vec) -> Morton {
        Morton(
            (shuffle(vec.x as u64) << 0) |
            (shuffle(vec.y as u64) << 1) |
            (shuffle(vec.z as u64) << 2)
        )
    }
}

impl From<Morton> for Vec {
    fn from(idx: Morton) -> Vec {
        Vec {
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

#[test]
fn test_encoding() {
    assert!(Morton::from(&Vec { x: 0, y: 0, z: 0 }) == Morton::from(0  as u64));
    assert!(Morton::from(&Vec { x: 1, y: 0, z: 0 }) == Morton::from(1  as u64));
    assert!(Morton::from(&Vec { x: 0, y: 1, z: 0 }) == Morton::from(2  as u64));
    assert!(Morton::from(&Vec { x: 1, y: 1, z: 0 }) == Morton::from(3  as u64));
    assert!(Morton::from(&Vec { x: 0, y: 0, z: 1 }) == Morton::from(4  as u64));
    assert!(Morton::from(&Vec { x: 1, y: 0, z: 1 }) == Morton::from(5  as u64));
    assert!(Morton::from(&Vec { x: 0, y: 1, z: 1 }) == Morton::from(6  as u64));
    assert!(Morton::from(&Vec { x: 1, y: 1, z: 1 }) == Morton::from(7  as u64));
    assert!(Morton::from(&Vec { x: 2, y: 0, z: 0 }) == Morton::from(8  as u64));
    assert!(Morton::from(&Vec { x: 0, y: 2, z: 0 }) == Morton::from(16 as u64));
    assert!(Morton::from(&Vec { x: 0, y: 0, z: 2 }) == Morton::from(32 as u64));
}

#[test]
fn test_decoding() {
    assert!(Vec { x: 0, y: 0, z: 0 } == Vec::from(Morton::from(0  as u64)));
    assert!(Vec { x: 1, y: 0, z: 0 } == Vec::from(Morton::from(1  as u64)));
    assert!(Vec { x: 0, y: 1, z: 0 } == Vec::from(Morton::from(2  as u64)));
    assert!(Vec { x: 1, y: 1, z: 0 } == Vec::from(Morton::from(3  as u64)));
    assert!(Vec { x: 0, y: 0, z: 1 } == Vec::from(Morton::from(4  as u64)));
    assert!(Vec { x: 1, y: 0, z: 1 } == Vec::from(Morton::from(5  as u64)));
    assert!(Vec { x: 0, y: 1, z: 1 } == Vec::from(Morton::from(6  as u64)));
    assert!(Vec { x: 1, y: 1, z: 1 } == Vec::from(Morton::from(7  as u64)));
    assert!(Vec { x: 2, y: 0, z: 0 } == Vec::from(Morton::from(8  as u64)));
    assert!(Vec { x: 0, y: 2, z: 0 } == Vec::from(Morton::from(16 as u64)));
    assert!(Vec { x: 0, y: 0, z: 2 } == Vec::from(Morton::from(32 as u64)));
}
