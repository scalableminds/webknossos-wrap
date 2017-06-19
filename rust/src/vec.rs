use std::cmp::{min, max, Ordering};
use ::Result;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec3 {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

#[derive(Copy, Clone, Debug)]
pub struct Box3 {
    min: Vec3,
    max: Vec3
}

impl Box3 {
    pub fn new(min: Vec3, max: Vec3) -> Result<Box3> {
        if max < min {
            println!("min = {:?}", min);
            println!("max = {:?}", max);
            return Err("Minimum and maximum are in conflict");
        }

        Ok(Box3 {
            min: min,
            max: max
        })
    }

    pub fn min(&self) -> Vec3 { self.min }
    pub fn max(&self) -> Vec3 { self.max }
    pub fn width(&self) -> Vec3 { self.max - self.min }
}

impl From<Vec3> for Box3 {
    fn from(max: Vec3) -> Box3 {
        Box3 {
            min: Vec3::from(0u32),
            max: max
        }
    }
}

impl Vec3 {
    pub fn is_zero(&self) -> bool {
        self == &Vec3::from(0u32)
    }

    pub fn is_cube_diagonal(&self) -> bool {
        self.x == self.y &&
        self.x == self.z &&
        self.y == self.z
    }

    pub fn is_power_of_two(&self) -> bool {
        self.x.is_power_of_two() &&
        self.y.is_power_of_two() &&
        self.z.is_power_of_two()
    }

    pub fn is_multiple_of(&self, other: Vec3) -> bool {
        self.rem(other).is_zero()
    }

    pub fn elem_max(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: max(self.x, other.x),
            y: max(self.y, other.y),
            z: max(self.z, other.z)
        }
    }

    pub fn elem_min(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: min(self.x, other.x),
            y: min(self.y, other.y),
            z: min(self.z, other.z)
        }
    }
}

// based on bluss' ndarray
macro_rules! impl_binary_op(
    ($trt:ident, $operator:tt, $mth:ident) => (

use std::ops::$trt;

impl $trt<Vec3> for Vec3 {
    type Output = Vec3;

    fn $mth(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x $operator rhs.x,
            y: self.y $operator rhs.y,
            z: self.z $operator rhs.z
        }
    }
}

impl $trt<u32> for Vec3 {
    type Output = Vec3;

    fn $mth(self, rhs: u32) -> Vec3 {
        Vec3 {
            x: self.x $operator rhs,
            y: self.y $operator rhs,
            z: self.z $operator rhs
        }
    }
}

impl $trt<Box3> for Box3 {
    type Output = Box3;

    fn $mth(self, rhs: Box3) -> Box3 {
        Box3 {
            min: self.min $operator rhs.min,
            max: self.max $operator rhs.max
        }
    }
}

impl $trt<Vec3> for Box3 {
    type Output = Box3;

    fn $mth(self, rhs: Vec3) -> Box3 {
        Box3 {
            min: self.min $operator rhs,
            max: self.max $operator rhs
        }
    }
}

impl $trt<u32> for Box3 {
    type Output = Box3;

    fn $mth(self, rhs: u32) -> Box3 {
        Box3 {
            min: self.min $operator rhs,
            max: self.max $operator rhs
        }
    }
}
    ) // macro_rules rule
); // macro_rules

impl_binary_op!(Add, +, add);
impl_binary_op!(Sub, -, sub);
impl_binary_op!(Mul, *, mul);
impl_binary_op!(Div, /, div);
impl_binary_op!(Rem, %, rem);
impl_binary_op!(Shl, <<, shl);
impl_binary_op!(Shr, >>, shr);

impl From<u32> for Vec3 {
    fn from(s: u32) -> Vec3 {
        Vec3 { x: s, y: s, z: s }
    }
}

impl PartialOrd for Vec3 {
    fn partial_cmp(&self, rhs: &Vec3) -> Option<Ordering> {
        let ords = [
            self.x.cmp(&rhs.x),
            self.y.cmp(&rhs.y),
            self.z.cmp(&rhs.z)
        ];

        let any_lt = ords.iter().any(|s| s == &Ordering::Less);
        let any_gt = ords.iter().any(|s| s == &Ordering::Greater);

        match (any_lt, any_gt) {
            (false, false) => Some(Ordering::Equal),
            (false, true)  => Some(Ordering::Greater),
            (true,  false) => Some(Ordering::Less),
            (true,  true)  => None
        }
    }
}
