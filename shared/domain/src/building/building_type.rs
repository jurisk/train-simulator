use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::industry_type::IndustryType;
use crate::building::station_type::StationType;
use crate::building::WithRelativeTileCoverage;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::track_type::TrackType;

// TODO HIGH: Split stations from industries, like you split out tracks before?
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum BuildingType {
    Station(StationType),
    Industry(IndustryType),
}

impl Debug for BuildingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildingType::Station(station_type) => write!(f, "S-{station_type:?}"),
            BuildingType::Industry(industry_type) => write!(f, "I-{industry_type:?}"),
        }
    }
}

impl BuildingType {
    #[must_use]
    pub fn track_types_at(self, relative_tile: TileCoordsXZ) -> Vec<TrackType> {
        match self {
            BuildingType::Industry(_) => vec![],
            BuildingType::Station(station_type) => station_type.track_types_at(relative_tile),
        }
    }
}

impl WithRelativeTileCoverage for BuildingType {
    fn relative_tiles_used(&self) -> TileCoverage {
        match self {
            BuildingType::Industry(industry_type) => industry_type.relative_tiles_used(),
            BuildingType::Station(station_type) => station_type.relative_tiles_used(),
        }
    }
}
