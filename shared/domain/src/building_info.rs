use serde::{Deserialize, Serialize};

use crate::building_type::BuildingType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::tile_track::TileTrack;
use crate::track_type::TrackType;
use crate::{BuildingId, PlayerId};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct BuildingInfo {
    pub owner_id:       PlayerId,
    pub building_id:    BuildingId,
    pub reference_tile: TileCoordsXZ,
    pub building_type:  BuildingType,
}

impl BuildingInfo {
    #[must_use]
    pub fn tile_tracks(&self) -> Vec<TileTrack> {
        let mut results = Vec::new();
        for relative_tile in self.building_type.relative_tiles_used().to_set() {
            for track_type in self.building_type.track_types_at(relative_tile) {
                results.push(TileTrack {
                    tile_coords_xz: self.reference_tile + relative_tile,
                    track_type,
                });
            }
        }
        results
    }

    #[must_use]
    pub fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        self.building_type
            .track_types_at(tile - self.reference_tile)
    }

    #[must_use]
    pub fn covers_tiles(self) -> TileCoverage {
        self.building_type
            .relative_tiles_used()
            .offset_by(self.reference_tile)
    }
}
