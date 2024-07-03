use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::tile_track::TileTrack;
use crate::track_type::TrackType;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum StationOrientation {
    NorthToSouth,
    EastToWest,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub struct StationType {
    pub orientation:     StationOrientation,
    pub platforms:       usize,
    pub length_in_tiles: usize,
}

impl StationType {
    #[must_use]
    pub const fn all() -> [Self; 2] {
        [
            StationType {
                orientation:     StationOrientation::NorthToSouth,
                platforms:       1,
                length_in_tiles: 4,
            },
            StationType {
                orientation:     StationOrientation::EastToWest,
                platforms:       1,
                length_in_tiles: 4,
            },
        ]
    }

    /// These are the last `TileTrack`-s in a station, so if a train is parked `about_to_exit` on
    /// one of these, it is properly parked at the station using all of its length.
    #[must_use]
    pub fn exit_tile_tracks(self, reference_tile: TileCoordsXZ) -> HashSet<TileTrack> {
        let mut results = HashSet::new();
        for platform in 0 .. self.platforms {
            match self.orientation {
                StationOrientation::NorthToSouth => {
                    let a = TileTrack {
                        tile_coords_xz: reference_tile + TileCoordsXZ::from_usizes(platform, 0),
                        track_type:     TrackType::NorthSouth,
                        pointing_in:    DirectionXZ::North,
                    };
                    let b = TileTrack {
                        tile_coords_xz: reference_tile
                            + TileCoordsXZ::from_usizes(platform, self.length_in_tiles - 1),
                        track_type:     TrackType::NorthSouth,
                        pointing_in:    DirectionXZ::South,
                    };
                    results.insert(a);
                    results.insert(b);
                },
                StationOrientation::EastToWest => {
                    let a = TileTrack {
                        tile_coords_xz: reference_tile + TileCoordsXZ::from_usizes(0, platform),
                        track_type:     TrackType::EastWest,
                        pointing_in:    DirectionXZ::West,
                    };
                    let b = TileTrack {
                        tile_coords_xz: reference_tile
                            + TileCoordsXZ::from_usizes(self.length_in_tiles - 1, platform),
                        track_type:     TrackType::EastWest,
                        pointing_in:    DirectionXZ::East,
                    };
                    results.insert(a);
                    results.insert(b);
                },
            }
        }
        results
    }

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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use shared_util::direction_xz::DirectionXZ;

    use crate::station_type::{StationOrientation, StationType};
    use crate::tile_coords_xz::TileCoordsXZ;
    use crate::tile_track::TileTrack;
    use crate::track_type::TrackType;

    #[test]
    fn exit_tile_tracks() {
        let station_type = StationType {
            orientation:     StationOrientation::NorthToSouth,
            platforms:       2,
            length_in_tiles: 3,
        };
        let reference_tile = TileCoordsXZ::from_usizes(10, 20);
        let actual = station_type.exit_tile_tracks(reference_tile);
        let expected: HashSet<TileTrack> = [
            TileTrack {
                tile_coords_xz: TileCoordsXZ::from_usizes(10, 20),
                track_type:     TrackType::NorthSouth,
                pointing_in:    DirectionXZ::North,
            },
            TileTrack {
                tile_coords_xz: TileCoordsXZ::from_usizes(10, 22),
                track_type:     TrackType::NorthSouth,
                pointing_in:    DirectionXZ::South,
            },
            TileTrack {
                tile_coords_xz: TileCoordsXZ::from_usizes(10, 20),
                track_type:     TrackType::NorthSouth,
                pointing_in:    DirectionXZ::North,
            },
            TileTrack {
                tile_coords_xz: TileCoordsXZ::from_usizes(10, 22),
                track_type:     TrackType::NorthSouth,
                pointing_in:    DirectionXZ::South,
            },
        ]
        .into_iter()
        .collect();
        assert_eq!(actual, expected);
    }
}
