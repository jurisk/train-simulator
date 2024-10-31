use std::f32::consts::SQRT_2;
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
use crate::transport::track_length::TrackLength;

// Later: Possibly rename to `ConnectionType` or something. And `TrackType` thus has multiple of these `ConnectionType`-s.
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum TrackType {
    NorthEast,
    NorthSouth,
    NorthWest,
    WestEast,
    SouthEast,
    SouthWest,
}

impl Debug for TrackType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackType::NorthEast => write!(f, "NE"),
            TrackType::NorthSouth => write!(f, "NS"),
            TrackType::NorthWest => write!(f, "NW"),
            TrackType::WestEast => write!(f, "WE"),
            TrackType::SouthEast => write!(f, "SE"),
            TrackType::SouthWest => write!(f, "SW"),
        }
    }
}

impl TrackType {
    const SQRT_2_DIV_2: f32 = SQRT_2 / 2.0;

    #[must_use]
    pub const fn all() -> [TrackType; 6] {
        [
            TrackType::NorthSouth,
            TrackType::WestEast,
            TrackType::NorthEast,
            TrackType::NorthWest,
            TrackType::SouthEast,
            TrackType::SouthWest,
        ]
    }

    #[must_use]
    #[expect(clippy::match_same_arms)]
    pub const fn from_directions(into: DirectionXZ, out: DirectionXZ) -> Option<TrackType> {
        match (into, out) {
            (DirectionXZ::North, DirectionXZ::South) => Some(TrackType::NorthSouth),
            (DirectionXZ::South, DirectionXZ::North) => Some(TrackType::NorthSouth),
            (DirectionXZ::East, DirectionXZ::West) => Some(TrackType::WestEast),
            (DirectionXZ::West, DirectionXZ::East) => Some(TrackType::WestEast),
            (DirectionXZ::North, DirectionXZ::East) => Some(TrackType::NorthEast),
            (DirectionXZ::East, DirectionXZ::North) => Some(TrackType::NorthEast),
            (DirectionXZ::North, DirectionXZ::West) => Some(TrackType::NorthWest),
            (DirectionXZ::West, DirectionXZ::North) => Some(TrackType::NorthWest),
            (DirectionXZ::South, DirectionXZ::East) => Some(TrackType::SouthEast),
            (DirectionXZ::East, DirectionXZ::South) => Some(TrackType::SouthEast),
            (DirectionXZ::South, DirectionXZ::West) => Some(TrackType::SouthWest),
            (DirectionXZ::West, DirectionXZ::South) => Some(TrackType::SouthWest),
            (DirectionXZ::North, DirectionXZ::North) => None,
            (DirectionXZ::East, DirectionXZ::East) => None,
            (DirectionXZ::South, DirectionXZ::South) => None,
            (DirectionXZ::West, DirectionXZ::West) => None,
        }
    }

    #[expect(clippy::match_same_arms)]
    #[must_use]
    pub const fn other_end(self, direction: DirectionXZ) -> Option<DirectionXZ> {
        match (self, direction) {
            (TrackType::NorthEast, DirectionXZ::North) => Some(DirectionXZ::East),
            (TrackType::NorthEast, DirectionXZ::East) => Some(DirectionXZ::North),
            (TrackType::NorthSouth, DirectionXZ::North) => Some(DirectionXZ::South),
            (TrackType::NorthSouth, DirectionXZ::South) => Some(DirectionXZ::North),
            (TrackType::NorthWest, DirectionXZ::North) => Some(DirectionXZ::West),
            (TrackType::NorthWest, DirectionXZ::West) => Some(DirectionXZ::North),
            (TrackType::WestEast, DirectionXZ::East) => Some(DirectionXZ::West),
            (TrackType::WestEast, DirectionXZ::West) => Some(DirectionXZ::East),
            (TrackType::SouthEast, DirectionXZ::South) => Some(DirectionXZ::East),
            (TrackType::SouthEast, DirectionXZ::East) => Some(DirectionXZ::South),
            (TrackType::SouthWest, DirectionXZ::South) => Some(DirectionXZ::West),
            (TrackType::SouthWest, DirectionXZ::West) => Some(DirectionXZ::South),
            _ => None,
        }
    }

    #[must_use]
    pub const fn other_end_unsafe(self, direction: DirectionXZ) -> DirectionXZ {
        match self.other_end(direction) {
            None => {
                panic!("Invalid track type");
            },
            Some(found) => found,
        }
    }

    #[must_use]
    pub const fn connections(self) -> [DirectionXZ; 2] {
        let (a, b) = self.connections_clockwise();
        [a, b]
    }

    #[must_use]
    pub const fn matching_direction(direction: DirectionXZ) -> [TrackType; 3] {
        match direction {
            DirectionXZ::North => {
                [
                    TrackType::NorthSouth,
                    TrackType::NorthEast,
                    TrackType::NorthWest,
                ]
            },
            DirectionXZ::East => {
                [
                    TrackType::WestEast,
                    TrackType::NorthEast,
                    TrackType::SouthEast,
                ]
            },
            DirectionXZ::South => {
                [
                    TrackType::NorthSouth,
                    TrackType::SouthEast,
                    TrackType::SouthWest,
                ]
            },
            DirectionXZ::West => {
                [
                    TrackType::WestEast,
                    TrackType::NorthWest,
                    TrackType::SouthWest,
                ]
            },
        }
    }

    #[must_use]
    pub const fn connections_clockwise(self) -> (DirectionXZ, DirectionXZ) {
        match self {
            TrackType::NorthEast => (DirectionXZ::North, DirectionXZ::East),
            TrackType::NorthSouth => (DirectionXZ::North, DirectionXZ::South),
            TrackType::NorthWest => (DirectionXZ::West, DirectionXZ::North),
            TrackType::WestEast => (DirectionXZ::East, DirectionXZ::West),
            TrackType::SouthEast => (DirectionXZ::East, DirectionXZ::South),
            TrackType::SouthWest => (DirectionXZ::South, DirectionXZ::West),
        }
    }

    #[must_use]
    pub const fn length(self) -> TrackLength {
        let result = match self {
            TrackType::NorthSouth | TrackType::WestEast => 1.0,
            TrackType::NorthEast
            | TrackType::NorthWest
            | TrackType::SouthEast
            | TrackType::SouthWest => Self::SQRT_2_DIV_2,
        };
        TrackLength::new(result)
    }
}

impl WithRelativeTileCoverage for TrackType {
    #[must_use]
    fn relative_tiles_used(&self) -> TileCoverage {
        TileCoverage::Single(TileCoordsXZ::ZERO)
    }
}

impl WithCostToBuild for TrackType {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        (
            IndustryType::ConstructionYard,
            CargoMap::from([(ResourceType::Steel, 0.1), (ResourceType::Timber, 0.1)]),
        )
    }
}
