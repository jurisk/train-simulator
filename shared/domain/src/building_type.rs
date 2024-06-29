use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::production_type::ProductionType;
use crate::station_type::StationType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::track_type::TrackType;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum BuildingType {
    Track(TrackType),
    Station(StationType),
    Production(ProductionType),
}

impl BuildingType {
    #[must_use]
    pub fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        match self {
            BuildingType::Track(track_type) => track_type.relative_tiles_used(),
            BuildingType::Production(production_type) => production_type.relative_tiles_used(),
            BuildingType::Station(station_type) => station_type.relative_tiles_used(),
        }
    }
}
