use std::fmt::{Debug, Formatter};

use bevy_math::Vec3;
use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::map_level::terrain::Terrain;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::track_type::TrackType;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct TileTrack {
    pub tile:        TileCoordsXZ,
    pub track_type:  TrackType,
    pub pointing_in: DirectionXZ,
}

impl Debug for TileTrack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}-{:?}-{:?}",
            self.tile, self.track_type, self.pointing_in
        )
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
        entry + direction * progress_within_tile.as_f32() * track_length
    }

    #[must_use]
    pub fn next_tile_coords(&self) -> TileCoordsXZ {
        self.tile + self.pointing_in
    }
}
