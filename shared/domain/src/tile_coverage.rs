use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TileCoverage {
    Empty,
    Single(TileCoordsXZ),
    Rectangular {
        north_west_inclusive: TileCoordsXZ,
        south_east_inclusive: TileCoordsXZ,
    },
}

impl TileCoverage {
    // Later:   Implement `Iterator` and `IntoIterator` properly - see https://dev.to/wrongbyte/implementing-iterator-and-intoiterator-in-rust-3nio.
    #[must_use]
    pub fn to_set(&self) -> HashSet<TileCoordsXZ> {
        match self {
            TileCoverage::Empty => HashSet::new(),
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
    pub fn contains(&self, tile: TileCoordsXZ) -> bool {
        match self {
            TileCoverage::Empty => false,
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
            TileCoverage::Empty => TileCoverage::Empty,
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
}
