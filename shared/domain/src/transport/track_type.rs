use std::f32::consts::SQRT_2;
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::building::WithRelativeTileCoverage;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::track_length::TrackLength;

// Later: Possibly rename to `ConnectionType` or something. And `TrackType` thus has multiple of these `ConnectionType`-s.
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash, Ord, PartialOrd)]
pub enum TrackType {
    NorthEast,
    NorthSouth,
    NorthWest,
    EastWest,
    SouthEast,
    SouthWest,
}

impl Debug for TrackType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackType::NorthEast => write!(f, "NE"),
            TrackType::NorthSouth => write!(f, "NS"),
            TrackType::NorthWest => write!(f, "NW"),
            TrackType::EastWest => write!(f, "EW"),
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
            TrackType::EastWest,
            TrackType::NorthEast,
            TrackType::NorthWest,
            TrackType::SouthEast,
            TrackType::SouthWest,
        ]
    }

    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn other_end(self, direction: DirectionXZ) -> Option<DirectionXZ> {
        match (self, direction) {
            (TrackType::NorthEast, DirectionXZ::North) => Some(DirectionXZ::East),
            (TrackType::NorthEast, DirectionXZ::East) => Some(DirectionXZ::North),
            (TrackType::NorthSouth, DirectionXZ::North) => Some(DirectionXZ::South),
            (TrackType::NorthSouth, DirectionXZ::South) => Some(DirectionXZ::North),
            (TrackType::NorthWest, DirectionXZ::North) => Some(DirectionXZ::West),
            (TrackType::NorthWest, DirectionXZ::West) => Some(DirectionXZ::North),
            (TrackType::EastWest, DirectionXZ::East) => Some(DirectionXZ::West),
            (TrackType::EastWest, DirectionXZ::West) => Some(DirectionXZ::East),
            (TrackType::SouthEast, DirectionXZ::South) => Some(DirectionXZ::East),
            (TrackType::SouthEast, DirectionXZ::East) => Some(DirectionXZ::South),
            (TrackType::SouthWest, DirectionXZ::South) => Some(DirectionXZ::West),
            (TrackType::SouthWest, DirectionXZ::West) => Some(DirectionXZ::South),
            _ => None,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn other_end_unsafe(self, direction: DirectionXZ) -> DirectionXZ {
        self.other_end(direction).unwrap_or_else(|| {
            panic!("Invalid track type {self:?} and direction {direction:?} combination")
        })
    }

    #[must_use]
    pub const fn connections(self) -> [DirectionXZ; 2] {
        let (a, b) = self.connections_clockwise();
        [a, b]
    }

    #[must_use]
    pub const fn connections_clockwise(self) -> (DirectionXZ, DirectionXZ) {
        match self {
            TrackType::NorthEast => (DirectionXZ::North, DirectionXZ::East),
            TrackType::NorthSouth => (DirectionXZ::North, DirectionXZ::South),
            TrackType::NorthWest => (DirectionXZ::West, DirectionXZ::North),
            TrackType::EastWest => (DirectionXZ::East, DirectionXZ::West),
            TrackType::SouthEast => (DirectionXZ::East, DirectionXZ::South),
            TrackType::SouthWest => (DirectionXZ::South, DirectionXZ::West),
        }
    }

    #[must_use]
    pub const fn length(self) -> TrackLength {
        let result = match self {
            TrackType::NorthSouth | TrackType::EastWest => 1.0,
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
