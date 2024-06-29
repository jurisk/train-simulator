use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum StationOrientation {
    NorthToSouth,
    EastToWest,
}

// TODO: Build some test stations in test setup
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct StationType {
    pub orientation:     StationOrientation,
    pub platforms:       usize,
    pub length_in_tiles: usize,
}

impl StationType {
    #[must_use]
    pub(crate) fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        let mut result = HashSet::new();
        match self.orientation {
            StationOrientation::NorthToSouth => {
                for x in 0 .. self.platforms {
                    for z in 0 .. self.length_in_tiles {
                        result.insert(TileCoordsXZ::from_usizes(x, z));
                    }
                }
            },
            StationOrientation::EastToWest => {
                for x in 0 .. self.length_in_tiles {
                    for z in 0 .. self.platforms {
                        result.insert(TileCoordsXZ::from_usizes(x, z));
                    }
                }
            },
        }
        result
    }
}
