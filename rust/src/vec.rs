use std::ops::{Add, Div};

#[derive(Copy, Clone, Debug)]
pub struct Vec {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

impl Vec {
    pub fn from_scalar(s: u32) -> Vec {
        Vec { x: s, y: s, z: s }
    }

    pub fn is_valid_offset(&self) -> bool {
        (self.x == self.y && self.x == self.z && self.y == self.z) &&
        (self.x == 0 || self.x.is_power_of_two())
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
}

impl From<u32> for Vec {
    fn from(s: u32) -> Vec {
        Vec::from_scalar(s)
    }
}

impl Add for Vec {
    type Output = Self;

    fn add(self, rhs: Vec) -> Self::Output {
        Vec {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

impl Div for Vec {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Vec {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}
