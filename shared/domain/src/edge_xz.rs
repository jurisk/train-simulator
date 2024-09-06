use shared_util::direction_xz::DirectionXZ;

use crate::tile_coords_xz::TileCoordsXZ;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EdgeXZ {
    Horizontal { west: TileCoordsXZ },
    Vertical { north: TileCoordsXZ },
}

impl EdgeXZ {
    #[must_use]
    pub fn from_tile_and_direction(tile: TileCoordsXZ, direction: DirectionXZ) -> Self {
        match direction {
            DirectionXZ::North => {
                EdgeXZ::Vertical {
                    north: tile + DirectionXZ::North,
                }
            },
            DirectionXZ::East => EdgeXZ::Horizontal { west: tile },
            DirectionXZ::South => EdgeXZ::Vertical { north: tile },
            DirectionXZ::West => {
                EdgeXZ::Horizontal {
                    west: tile + DirectionXZ::West,
                }
            },
        }
    }

    #[must_use]
    pub fn both_tiles_and_directions(self) -> [(TileCoordsXZ, DirectionXZ); 2] {
        match self {
            EdgeXZ::Horizontal { west } => {
                [
                    (west, DirectionXZ::East),
                    (west + DirectionXZ::East, DirectionXZ::West),
                ]
            },
            EdgeXZ::Vertical { north } => {
                [
                    (north, DirectionXZ::South),
                    (north + DirectionXZ::South, DirectionXZ::North),
                ]
            },
        }
    }

    #[must_use]
    pub fn for_tile(tile: TileCoordsXZ) -> [EdgeXZ; 4] {
        [
            EdgeXZ::from_tile_and_direction(tile, DirectionXZ::North),
            EdgeXZ::from_tile_and_direction(tile, DirectionXZ::East),
            EdgeXZ::from_tile_and_direction(tile, DirectionXZ::South),
            EdgeXZ::from_tile_and_direction(tile, DirectionXZ::West),
        ]
    }

    #[must_use]
    pub fn ordered_tiles(self) -> (TileCoordsXZ, TileCoordsXZ) {
        match self {
            EdgeXZ::Horizontal { west } => (west, west + DirectionXZ::East),
            EdgeXZ::Vertical { north } => (north, north + DirectionXZ::South),
        }
    }

    #[expect(clippy::if_same_then_else)]
    #[must_use]
    pub fn common_tile(a: EdgeXZ, b: EdgeXZ) -> Option<TileCoordsXZ> {
        let (a_0, a_1) = a.ordered_tiles();
        let (b_0, b_1) = b.ordered_tiles();
        if a == b {
            None // That means there are two common tiles, not a tile.
        } else if a_0 == b_0 {
            Some(a_0)
        } else if a_0 == b_1 {
            Some(a_0)
        } else if a_1 == b_0 {
            Some(a_1)
        } else if a_1 == b_1 {
            Some(a_1)
        } else {
            None
        }
    }
}
