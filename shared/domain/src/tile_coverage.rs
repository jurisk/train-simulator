use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TileCoverage {
    Single(TileCoordsXZ),
    Multiple(HashSet<TileCoordsXZ>),
}

impl TileCoverage {
    #[must_use]
    pub fn to_set(&self) -> HashSet<TileCoordsXZ> {
        match self {
            TileCoverage::Single(tile) => HashSet::from([*tile]),
            TileCoverage::Multiple(tiles) => tiles.clone(),
        }
    }

    #[must_use]
    pub fn contains(&self, tile: TileCoordsXZ) -> bool {
        match self {
            TileCoverage::Single(single_tile) => *single_tile == tile,
            TileCoverage::Multiple(tiles) => tiles.contains(&tile),
        }
    }
}
