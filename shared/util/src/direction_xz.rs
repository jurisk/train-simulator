use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::coords_xz::CoordsXZ;

// SW = X axis, smaller is W, larger is E
// NS = Z axis, smaller is N, larger is S
// Thus smallest coords are NW, and largest are SE.
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DirectionXZ {
    North,
    East,
    South,
    West,
}

impl Debug for DirectionXZ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectionXZ::North => write!(f, "N"),
            DirectionXZ::East => write!(f, "E"),
            DirectionXZ::South => write!(f, "S"),
            DirectionXZ::West => write!(f, "W"),
        }
    }
}

impl DirectionXZ {
    #[must_use]
    pub const fn cardinal() -> [Self; 4] {
        [Self::North, Self::East, Self::South, Self::West]
    }

    #[must_use]
    pub fn reverse(self) -> Self {
        match self {
            DirectionXZ::North => DirectionXZ::South,
            DirectionXZ::East => DirectionXZ::West,
            DirectionXZ::South => DirectionXZ::North,
            DirectionXZ::West => DirectionXZ::East,
        }
    }
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
