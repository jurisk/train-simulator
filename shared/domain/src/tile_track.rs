use std::fmt::{Debug, Formatter};

use bevy_math::Vec3;
use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::terrain::Terrain;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::track_type::TrackType;
use crate::transport_info::ProgressWithinTile;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct TileTrack {
    pub tile_coords_xz: TileCoordsXZ,
    pub track_type:     TrackType,
    pub pointing_in:    DirectionXZ,
}

impl Debug for TileTrack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.tile_coords_xz, self.track_type)
    }
}

impl TileTrack {
    #[must_use]
    pub fn progress_coordinates(
        self,
        progress_within_tile: ProgressWithinTile,
        terrain: &Terrain,
    ) -> Vec3 {
        let (entry, exit) = terrain.entry_and_exit(self);
        let track_length = (exit - entry).length();
        let direction = (exit - entry).normalize();
        entry + direction * progress_within_tile.progress() * track_length
    }
}
