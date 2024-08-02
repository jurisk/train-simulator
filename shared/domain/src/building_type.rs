use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::production_type::ProductionType;
use crate::station_type::StationType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::track_type::TrackType;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum BuildingType {
    Track(TrackType),
    Station(StationType),
    /* TODO: It's not really production, but is it `Industry`? Or something else? */
    Production(ProductionType),
}

impl Debug for BuildingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildingType::Track(track_type) => write!(f, "T-{track_type:?}"),
            BuildingType::Station(station_type) => write!(f, "S-{station_type:?}"),
            BuildingType::Production(production_type) => write!(f, "P-{production_type:?}"),
        }
    }
}

impl BuildingType {
    // Note:    Currently, this could return `Option`, but we may want to refactor the track tiles to have
    //          multiple tracks in the same "building", so a `Vec` may be helpful then.
    #[must_use]
    pub fn track_types_at(self, relative_tile: TileCoordsXZ) -> Vec<TrackType> {
        match self {
            BuildingType::Track(track_type) => vec![track_type],
            BuildingType::Production(_) => vec![],
            BuildingType::Station(station_type) => {
                if station_type.relative_tiles_used().contains(relative_tile) {
                    vec![station_type.track_type()]
                } else {
                    vec![]
                }
            },
        }
    }

    #[must_use]
    pub fn relative_tiles_used(self) -> TileCoverage {
        match self {
            BuildingType::Track(track_type) => track_type.relative_tiles_used(),
            BuildingType::Production(production_type) => production_type.relative_tiles_used(),
            BuildingType::Station(station_type) => station_type.relative_tiles_used(),
        }
    }
}
