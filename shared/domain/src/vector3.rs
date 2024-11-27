use std::ops::{AddAssign, Mul};

use bevy_math::Vec3;
use serde::{Deserialize, Serialize};

// Later: Do we really need this separate from Bevy/glam Vec3? But we do if we want to avoid floats and have a more granular, more predictable data model.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<Vec3> for Vector3 {
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<Vector3> for Vec3 {
    fn from(value: Vector3) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
