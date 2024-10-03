use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::{TileCoordsXZ, TileDistance};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TileCoverage {
    Single(TileCoordsXZ),
    Rectangular {
        north_west_inclusive: TileCoordsXZ,
        south_east_inclusive: TileCoordsXZ,
    },
}

impl TileCoverage {
    #[must_use]
    pub fn single_at_zero() -> Self {
        TileCoverage::Single(TileCoordsXZ::ZERO)
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn rectangular_odd(
        center: TileCoordsXZ,
        size_x: TileDistance,
        size_z: TileDistance,
    ) -> Self {
        assert_eq!(size_x % 2, 1, "size_x must be odd");
        assert_eq!(size_z % 2, 1, "size_z must be odd");
        TileCoverage::Rectangular {
            north_west_inclusive: TileCoordsXZ::new(center.x - size_x / 2, center.z - size_z / 2),
            south_east_inclusive: TileCoordsXZ::new(center.x + size_x / 2, center.z + size_z / 2),
        }
    }

    #[must_use]
    pub fn extend(&self, diff: TileDistance) -> Self {
        match self {
            TileCoverage::Single(tile) => {
                TileCoverage::Rectangular {
                    north_west_inclusive: TileCoordsXZ::new(tile.x - diff, tile.z - diff),
                    south_east_inclusive: TileCoordsXZ::new(tile.x + diff, tile.z + diff),
                }
            },
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                TileCoverage::Rectangular {
                    north_west_inclusive: TileCoordsXZ::new(
                        north_west_inclusive.x - diff,
                        north_west_inclusive.z - diff,
                    ),
                    south_east_inclusive: TileCoordsXZ::new(
                        south_east_inclusive.x + diff,
                        south_east_inclusive.z + diff,
                    ),
                }
            },
        }
    }

    // Later:   Implement `Iterator` and `IntoIterator` properly - see https://dev.to/wrongbyte/implementing-iterator-and-intoiterator-in-rust-3nio.
    #[must_use]
    pub fn to_set(&self) -> HashSet<TileCoordsXZ> {
        match self {
            TileCoverage::Single(tile) => HashSet::from([*tile]),
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                let mut results = HashSet::new();
                for x in north_west_inclusive.x ..= south_east_inclusive.x {
                    for z in north_west_inclusive.z ..= south_east_inclusive.z {
                        results.insert(TileCoordsXZ::new(x, z));
                    }
                }
                results
            },
        }
    }

    #[must_use]
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.to_set().len()
    }

    #[must_use]
    pub fn contains(&self, tile: TileCoordsXZ) -> bool {
        match self {
            TileCoverage::Single(single_tile) => *single_tile == tile,
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                tile.x >= north_west_inclusive.x
                    && tile.x <= south_east_inclusive.x
                    && tile.z >= north_west_inclusive.z
                    && tile.z <= south_east_inclusive.z
            },
        }
    }

    #[must_use]
    pub fn offset_by(self, tile: TileCoordsXZ) -> Self {
        match self {
            TileCoverage::Single(single_tile) => TileCoverage::Single(single_tile + tile),
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                TileCoverage::Rectangular {
                    north_west_inclusive: north_west_inclusive + tile,
                    south_east_inclusive: south_east_inclusive + tile,
                }
            },
        }
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        for a in self.to_set() {
            for b in other.to_set() {
                if a == b {
                    return true;
                }
            }
        }
        false
    }

    // TODO: This gets called often enough that you should optimise it
    #[must_use]
    pub fn manhattan_distance_between_closest_tiles(
        a: &TileCoverage,
        b: &TileCoverage,
    ) -> TileDistance {
        let mut result = i32::MAX;
        for a in a.to_set() {
            for b in b.to_set() {
                let distance = a.manhattan_distance(b);
                result = result.min(distance);
            }
        }
        result
    }
}
