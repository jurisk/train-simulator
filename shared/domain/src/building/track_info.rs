use serde::{Deserialize, Serialize};

use crate::edge_xz::EdgeXZ;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_type::TrackType;
use crate::{PlayerId, TrackId};

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
            id: TrackId::random(),
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
    pub fn owner_id(&self) -> PlayerId {
        self.owner_id
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
