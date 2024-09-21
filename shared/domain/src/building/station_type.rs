use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::building::WithRelativeTileCoverage;
use crate::building::building_info::WithCostToBuild;
use crate::building::industry_type::IndustryType;
use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_type::TrackType;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub struct PlatformIndex(usize);

impl PlatformIndex {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum StationOrientation {
    NorthToSouth,
    EastToWest,
}

impl Debug for StationOrientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StationOrientation::NorthToSouth => write!(f, "NS"),
            StationOrientation::EastToWest => write!(f, "EW"),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub struct StationType {
    pub orientation:     StationOrientation,
    pub platforms:       usize,
    pub length_in_tiles: usize,
}

impl Debug for StationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}-{}-{}",
            self.orientation, self.platforms, self.length_in_tiles
        )
    }
}

impl StationType {
    pub const EW_1_4: StationType = StationType {
        orientation:     StationOrientation::EastToWest,
        platforms:       1,
        length_in_tiles: 4,
    };
    pub const NS_1_4: StationType = StationType {
        orientation:     StationOrientation::NorthToSouth,
        platforms:       1,
        length_in_tiles: 4,
    };

    #[must_use]
    pub const fn all() -> [Self; 2] {
        [Self::NS_1_4, Self::EW_1_4]
    }

    /// These are the last `TileTrack`-s in a station, so if a train is parked `about_to_exit` on
    /// one of these, it is properly parked at the station using all of its length.
    /// Think about how to return the directions and platforms in a more structured way.
    #[must_use]
    pub fn exit_tile_tracks(self, reference_tile: TileCoordsXZ) -> Vec<(PlatformIndex, TileTrack)> {
        let mut results = vec![];
        for platform in 0 .. self.platforms {
            let platform_index = PlatformIndex(platform);
            match self.orientation {
                StationOrientation::NorthToSouth => {
                    let a = TileTrack {
                        tile:        reference_tile + TileCoordsXZ::from_usizes(platform, 0),
                        track_type:  TrackType::NorthSouth,
                        pointing_in: DirectionXZ::North,
                    };
                    let b = TileTrack {
                        tile:        reference_tile
                            + TileCoordsXZ::from_usizes(platform, self.length_in_tiles - 1),
                        track_type:  TrackType::NorthSouth,
                        pointing_in: DirectionXZ::South,
                    };
                    results.push((platform_index, a));
                    results.push((platform_index, b));
                },
                StationOrientation::EastToWest => {
                    let a = TileTrack {
                        tile:        reference_tile + TileCoordsXZ::from_usizes(0, platform),
                        track_type:  TrackType::EastWest,
                        pointing_in: DirectionXZ::West,
                    };
                    let b = TileTrack {
                        tile:        reference_tile
                            + TileCoordsXZ::from_usizes(self.length_in_tiles - 1, platform),
                        track_type:  TrackType::EastWest,
                        pointing_in: DirectionXZ::East,
                    };
                    results.push((platform_index, a));
                    results.push((platform_index, b));
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
    pub fn track_types_at(self, relative_tile: TileCoordsXZ) -> Vec<TrackType> {
        if self.relative_tiles_used().contains(relative_tile) {
            vec![self.track_type()]
        } else {
            vec![]
        }
    }
}

impl WithRelativeTileCoverage for StationType {
    #[must_use]
    fn relative_tiles_used(&self) -> TileCoverage {
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

impl WithCostToBuild for StationType {
    #[expect(clippy::cast_precision_loss)]
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        let (industry_type, mut cargo_map) = self.track_type().cost_to_build();
        cargo_map += &CargoMap::single(ResourceType::Concrete, 0.1);
        cargo_map *= self.relative_tiles_used().len() as f32;
        (industry_type, cargo_map)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use shared_util::direction_xz::DirectionXZ;

    use crate::building::station_type::{StationOrientation, StationType};
    use crate::tile_coords_xz::TileCoordsXZ;
    use crate::transport::tile_track::TileTrack;
    use crate::transport::track_type::TrackType;

    #[test]
    fn exit_tile_tracks() {
        let station_type = StationType {
            orientation:     StationOrientation::NorthToSouth,
            platforms:       2,
            length_in_tiles: 3,
        };
        let reference_tile = TileCoordsXZ::from_usizes(10, 20);
        let actual = station_type
            .exit_tile_tracks(reference_tile)
            .into_iter()
            .map(|(_, tile_track)| tile_track)
            .collect::<HashSet<_>>();
        let expected: HashSet<TileTrack> = [
            TileTrack {
                tile:        TileCoordsXZ::from_usizes(10, 20),
                track_type:  TrackType::NorthSouth,
                pointing_in: DirectionXZ::North,
            },
            TileTrack {
                tile:        TileCoordsXZ::from_usizes(10, 22),
                track_type:  TrackType::NorthSouth,
                pointing_in: DirectionXZ::South,
            },
            TileTrack {
                tile:        TileCoordsXZ::from_usizes(11, 20),
                track_type:  TrackType::NorthSouth,
                pointing_in: DirectionXZ::North,
            },
            TileTrack {
                tile:        TileCoordsXZ::from_usizes(11, 22),
                track_type:  TrackType::NorthSouth,
                pointing_in: DirectionXZ::South,
            },
        ]
        .into_iter()
        .collect();
        assert_eq!(actual, expected);
    }
}
