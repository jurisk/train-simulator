use serde::{Deserialize, Serialize};

use crate::building_type::BuildingType;
use crate::cargo_map::CargoMap;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::tile_track::TileTrack;
use crate::track_type::TrackType;
use crate::{BuildingId, PlayerId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildingInfo {
    static_info:  BuildingStaticInfo,
    // TODO: Not all building types have dynamic info, and it can differ between building types... think of a better design.
    dynamic_info: BuildingDynamicInfo,
}

impl BuildingInfo {
    #[must_use]
    pub fn new(
        owner_id: PlayerId,
        building_id: BuildingId,
        reference_tile: TileCoordsXZ,
        building_type: BuildingType,
    ) -> Self {
        Self {
            static_info:  BuildingStaticInfo {
                owner_id,
                building_id,
                reference_tile,
                building_type,
            },
            dynamic_info: BuildingDynamicInfo {
                cargo: CargoMap::new(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct BuildingStaticInfo {
    owner_id:       PlayerId,
    building_id:    BuildingId,
    reference_tile: TileCoordsXZ,
    building_type:  BuildingType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildingDynamicInfo {
    cargo: CargoMap,
}

impl BuildingInfo {
    #[must_use]
    pub fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id
    }

    #[must_use]
    pub fn building_id(&self) -> BuildingId {
        self.static_info.building_id
    }

    #[must_use]
    pub fn building_type(&self) -> BuildingType {
        self.static_info.building_type
    }

    #[must_use]
    pub(crate) fn reference_tile(&self) -> TileCoordsXZ {
        self.static_info.reference_tile
    }

    #[must_use]
    pub fn tile_tracks(&self) -> Vec<TileTrack> {
        let mut results = Vec::new();
        for relative_tile in self.building_type().relative_tiles_used().to_set() {
            for track_type in self.building_type().track_types_at(relative_tile) {
                for pointing_in in track_type.connections() {
                    results.push(TileTrack {
                        tile_coords_xz: self.reference_tile() + relative_tile,
                        track_type,
                        pointing_in,
                    });
                }
            }
        }
        results
    }

    #[must_use]
    pub(crate) fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        self.building_type()
            .track_types_at(tile - self.reference_tile())
    }

    #[must_use]
    pub fn covers_tiles(&self) -> TileCoverage {
        self.building_type()
            .relative_tiles_used()
            .offset_by(self.reference_tile())
    }
}
