use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::tile_track::TileTrack;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportLocation {
    pub tile_path:            Vec<TileTrack>, /* Which tile is it on now, and which tiles has it been on - only as much as to cover the vehicle's length */
    pub progress_within_tile: ProgressWithinTile,
}

impl Debug for TransportLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.tile_path[0], self.progress_within_tile)
    }
}

impl TransportLocation {
    #[must_use]
    pub fn new(tile_path: Vec<TileTrack>, progress_within_tile: ProgressWithinTile) -> Self {
        Self {
            tile_path,
            progress_within_tile,
        }
    }

    #[must_use]
    pub fn entering_from(&self) -> DirectionXZ {
        let current_tile_track = self.tile_path[0];
        current_tile_track
            .track_type
            .other_end_unsafe(current_tile_track.pointing_in)
    }

    #[must_use]
    pub fn progress_within_tile(&self) -> ProgressWithinTile {
        self.progress_within_tile
    }

    #[must_use]
    pub fn tile_path(&self) -> Vec<TileTrack> {
        self.tile_path.clone()
    }

    #[must_use]
    pub fn next_tile_in_path(&self) -> TileTrack {
        self.tile_path[0]
    }
}
