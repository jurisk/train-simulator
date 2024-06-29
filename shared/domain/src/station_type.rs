use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::track_type::TrackType;

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
    pub fn track_type(self) -> TrackType {
        match self.orientation {
            StationOrientation::NorthToSouth => TrackType::NorthSouth,
            StationOrientation::EastToWest => TrackType::EastWest,
        }
    }

    #[must_use]
    pub fn relative_tiles_used(self) -> TileCoverage {
        match self.orientation {
            StationOrientation::NorthToSouth => {
                TileCoverage::Rectangular {
                    north_west_inclusive: TileCoordsXZ::from_usizes(0, 0),
                    south_east_inclusive: TileCoordsXZ::from_usizes(
                        self.platforms - 1,
                        self.length_in_tiles - 1,
                    ),
                }
            },
            StationOrientation::EastToWest => {
                TileCoverage::Rectangular {
                    north_west_inclusive: TileCoordsXZ::from_usizes(0, 0),
                    south_east_inclusive: TileCoordsXZ::from_usizes(
                        self.length_in_tiles - 1,
                        self.platforms - 1,
                    ),
                }
            },
        }
    }
}
