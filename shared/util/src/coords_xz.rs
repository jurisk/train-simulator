#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use std::fmt::{Debug, Formatter};
use std::ops::{Add, Mul, Sub};

use serde::{Deserialize, Serialize};

use crate::direction_xz::DirectionXZ;

// Note - the coordinates are i32 because they can be negative when derived from DirectionXZ
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CoordsXZ {
    pub x: i32,
    pub z: i32,
}

impl CoordsXZ {
    pub const ZERO: CoordsXZ = CoordsXZ { x: 0, z: 0 };

    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[must_use]
    pub fn from_usizes(x: usize, z: usize) -> Self {
        Self {
            x: x as i32,
            z: z as i32,
        }
    }
}

impl Debug for CoordsXZ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}

impl Add<CoordsXZ> for CoordsXZ {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            z: self.z + rhs.z,
        }
    }
}

impl Add<DirectionXZ> for CoordsXZ {
    type Output = Self;

    fn add(self, rhs: DirectionXZ) -> Self::Output {
        self + CoordsXZ::from(rhs)
    }
}

impl Sub<CoordsXZ> for CoordsXZ {
    type Output = Self;

    fn sub(self, rhs: CoordsXZ) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<i32> for CoordsXZ {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            z: self.z * rhs,
        }
    }
}
