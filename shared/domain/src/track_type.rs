use std::collections::HashSet;
use std::f32::consts::SQRT_2;
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;

// Later: Possibly rename to `ConnectionType` or something. And `TrackType` thus has multiple of these `ConnectionType`-s.
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
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

    #[allow(clippy::match_same_arms, clippy::missing_panics_doc)]
    #[must_use]
    pub fn other_end(self, direction: DirectionXZ) -> DirectionXZ {
        match (self, direction) {
            (TrackType::NorthEast, DirectionXZ::North) => DirectionXZ::East,
            (TrackType::NorthEast, DirectionXZ::East) => DirectionXZ::North,
            (TrackType::NorthSouth, DirectionXZ::North) => DirectionXZ::South,
            (TrackType::NorthSouth, DirectionXZ::South) => DirectionXZ::North,
            (TrackType::NorthWest, DirectionXZ::North) => DirectionXZ::West,
            (TrackType::NorthWest, DirectionXZ::West) => DirectionXZ::North,
            (TrackType::EastWest, DirectionXZ::East) => DirectionXZ::West,
            (TrackType::EastWest, DirectionXZ::West) => DirectionXZ::East,
            (TrackType::SouthEast, DirectionXZ::South) => DirectionXZ::East,
            (TrackType::SouthEast, DirectionXZ::East) => DirectionXZ::South,
            (TrackType::SouthWest, DirectionXZ::South) => DirectionXZ::West,
            (TrackType::SouthWest, DirectionXZ::West) => DirectionXZ::South,
            _ => {
                panic!("Invalid track type {self:?} and direction {direction:?} combination",)
            },
        }
    }

    #[must_use]
    pub fn relative_tiles_used(self) -> TileCoverage {
        TileCoverage::Single(TileCoordsXZ::ZERO)
    }

    #[must_use]
    pub fn connections(self) -> HashSet<DirectionXZ> {
        let (a, b) = self.connections_clockwise();
        HashSet::from([a, b])
    }

    #[must_use]
    pub fn connections_clockwise(self) -> (DirectionXZ, DirectionXZ) {
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
    pub fn length_in_tiles(self) -> f32 {
        match self {
            TrackType::NorthSouth | TrackType::EastWest => 1.0,
            TrackType::NorthEast
            | TrackType::NorthWest
            | TrackType::SouthEast
            | TrackType::SouthWest => SQRT_2 / 2.0,
        }
    }
}
