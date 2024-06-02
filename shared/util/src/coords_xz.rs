use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
