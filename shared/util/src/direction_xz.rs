use crate::coords_xz::CoordsXZ;

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
