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

impl From<u32> for Vec {
    fn from(s: u32) -> Vec {
        Vec { x: s, y: s, z: s }
    }
}

impl<'a> Add<&'a Vec> for Vec {
    type Output = Vec;
    fn add(self, rhs: &Vec) -> Self::Output {
        Vec { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Add<u32> for Vec {
    type Output = Vec;
    fn add(self, rhs: u32) -> Self::Output {
        Vec { x: self.x + rhs, y: self.y + rhs, z: self.z + rhs }
    }
}

impl Sub<u32> for Vec {
    type Output = Vec;
    fn sub(self, rhs: u32) -> Self::Output {
        Vec { x: self.x - rhs, y: self.y - rhs, z: self.z - rhs }
    }
}

impl Mul<u32> for Vec {
    type Output = Vec;
    fn mul(self, rhs: u32) -> Self::Output {
        Vec { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Div<u32> for Vec {
    type Output = Self;
    fn div(self, rhs: u32) -> Self::Output {
        Vec { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl Shl<u32> for Vec {
    type Output = Self;
    fn shl(self, rhs: u32) -> Self:: Output {
        Vec { x: self.x << rhs, y: self.y << rhs, z: self.z << rhs }
    }
}

impl Shr<u32> for Vec {
    type Output = Self;
    fn shr(self, rhs: u32) -> Self:: Output {
        Vec { x: self.x >> rhs, y: self.y >> rhs, z: self.z >> rhs }
    }
}
