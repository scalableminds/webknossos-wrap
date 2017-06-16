use std::ops::{Add, Sub, Mul, Div, Shl, Shr};
use ::Result;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec3 {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

pub struct Box3 {
    min: Vec3,
    max: Vec3
}

impl Box3 {
    pub fn new(min: Vec3, max: Vec3) -> Result<Box3> {
        if min.x > max.x || min.y > max.y || min.z > max.z {
            Err("Minimum and maximum vectors are conflicting")
        } else {
            Ok(Box3 { min: min, max: max })
        }
    }

    pub fn min(&self) -> Vec3 { self.min }
    pub fn max(&self) -> Vec3 { self.max }
    pub fn width(&self) -> Vec3 { self.max - self.min }
}

impl Vec3 {
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

    pub fn is_larger_equal_than(&self, other: &Vec3) -> bool {
        self.x >= other.x &&
        self.y >= other.y &&
        self.z >= other.z
    }

    pub fn is_multiple_of(&self, other: &Vec3) -> bool {
        self.x % other.x == 0 &&
        self.y % other.y == 0 &&
        self.z % other.z == 0
    }
}

// based on bluss' ndarray
macro_rules! impl_binary_op(
    ($trt:ident, $operator:tt, $mth:ident) => (

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
    ) // macro_rules rule
); // macro_rules

impl_binary_op!(Add, +, add);
impl_binary_op!(Sub, -, sub);
impl_binary_op!(Mul, *, mul);
impl_binary_op!(Div, /, div);
impl_binary_op!(Shl, <<, shl);
impl_binary_op!(Shr, >>, shr);

impl From<u32> for Vec3 {
    fn from(s: u32) -> Vec3 {
        Vec3 { x: s, y: s, z: s }
    }
}
