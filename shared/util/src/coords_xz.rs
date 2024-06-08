use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CoordsXZ {
    pub x: usize,
    pub z: usize,
}

impl CoordsXZ {
    #[must_use]
    pub fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }
}

impl Debug for CoordsXZ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}
