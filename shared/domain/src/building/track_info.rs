use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::track_type::TrackType;
use crate::{PlayerId, TrackId};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct TrackInfo {
    id:             TrackId,
    pub owner_id:   PlayerId,
    pub tile:       TileCoordsXZ,
    pub track_type: TrackType,
}

impl TrackInfo {
    #[must_use]
    pub fn new(
        track_id: TrackId,
        owner_id: PlayerId,
        tile: TileCoordsXZ,
        track_type: TrackType,
    ) -> Self {
        Self {
            id: track_id,
            owner_id,
            tile,
            track_type,
        }
    }

    #[must_use]
    pub fn id(&self) -> TrackId {
        self.id
    }
}
