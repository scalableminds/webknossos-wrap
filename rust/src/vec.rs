use std::ops::{Add, Sub, Mul, Div, Shl, Shr};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

impl Vec {
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

    pub fn is_larger_equal_than(&self, other: &Vec) -> bool {
        self.x >= other.x &&
        self.y >= other.y &&
        self.z >= other.z
    }

    pub fn is_multiple_of(&self, other: &Vec) -> bool {
        self.x % other.x == 0 &&
        self.y % other.y == 0 &&
        self.z % other.z == 0
    }
}

// based on bluss' ndarray
macro_rules! impl_binary_op(
    ($trt:ident, $operator:tt, $mth:ident) => (

impl $trt<Vec> for Vec {
    type Output = Vec;

    fn $mth(self, rhs: Vec) -> Vec {
        Vec {
            x: self.x $operator rhs.x,
            y: self.y $operator rhs.y,
            z: self.z $operator rhs.z
        }
    }
}

impl $trt<u32> for Vec {
    type Output = Vec;

    fn $mth(self, rhs: u32) -> Vec {
        Vec {
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

impl From<u32> for Vec {
    fn from(s: u32) -> Vec {
        Vec { x: s, y: s, z: s }
    }
}
