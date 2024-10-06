use serde::{Deserialize, Serialize};

use crate::building::building_info::{WithCostToBuild, WithOwner, WithTileCoverage};
use crate::building::industry_type::IndustryType;
use crate::cargo_map::CargoMap;
use crate::edge_xz::EdgeXZ;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_type::TrackType;
use crate::{PlayerId, TrackId};

// Later:   This is somewhat awkward as it's really just a DTO at this point and our internal
//          representation is different - perhaps it can be refactored to something more elegant.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TrackInfo {
    id:             TrackId,
    owner_id:       PlayerId,
    pub tile:       TileCoordsXZ,
    pub track_type: TrackType,
}

impl TrackInfo {
    #[must_use]
    pub fn new(owner_id: PlayerId, tile: TileCoordsXZ, track_type: TrackType) -> Self {
        Self {
            id: TrackId::new(tile, track_type),
            owner_id,
            tile,
            track_type,
        }
    }

    #[must_use]
    pub fn from_tile_track(owner_id: PlayerId, tile_track: TileTrack) -> Self {
        Self::new(owner_id, tile_track.tile, tile_track.track_type)
    }

    #[must_use]
    pub fn id(&self) -> TrackId {
        self.id
    }

    #[must_use]
    pub fn edges_clockwise(&self) -> [EdgeXZ; 2] {
        let (a, b) = self.track_type.connections_clockwise();
        [
            EdgeXZ::from_tile_and_direction(self.tile, a),
            EdgeXZ::from_tile_and_direction(self.tile, b),
        ]
    }
}

impl WithOwner for TrackInfo {
    fn owner_id(&self) -> PlayerId {
        self.owner_id
    }
}

impl WithCostToBuild for TrackInfo {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        self.track_type.cost_to_build()
    }
}

impl WithTileCoverage for TrackInfo {
    fn covers_tiles(&self) -> TileCoverage {
        TileCoverage::Single(self.tile)
    }
}
