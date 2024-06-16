use serde::{Deserialize, Serialize};

use crate::coords_xz::CoordsXZ;

// SW = X axis, smaller is W, larger is E
// NS = Z axis, smaller is N, larger is S
#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum DirectionXZ {
    North,
    East,
    South,
    West,
}

impl From<DirectionXZ> for CoordsXZ {
    fn from(direction: DirectionXZ) -> Self {
        match direction {
            DirectionXZ::North => CoordsXZ::new(0, -1),
            DirectionXZ::East => CoordsXZ::new(1, 0),
            DirectionXZ::South => CoordsXZ::new(0, 1),
            DirectionXZ::West => CoordsXZ::new(-1, 0),
        }
    }
}
