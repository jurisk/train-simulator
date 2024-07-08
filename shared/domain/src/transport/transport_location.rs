use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::tile_track::TileTrack;
use crate::transport::transport_type::TransportType;

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
            .other_end(current_tile_track.pointing_in)
    }

    #[must_use]
    pub fn progress_within_tile(&self) -> ProgressWithinTile {
        self.progress_within_tile
    }

    #[must_use]
    pub fn tile_path(&self) -> Vec<TileTrack> {
        self.tile_path.clone()
    }

    // TODO HIGH: Make private
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::items_after_statements
    )]
    pub(crate) fn perform_jump(
        &mut self,
        transport_type: &TransportType,
        next_tile_track: TileTrack,
    ) {
        self.tile_path.insert(0, next_tile_track);

        // Later: We are rather crudely sometimes removing the last element when we are inserting an
        // element.
        // This means - depending on `HEURISTIC_COEF` - that sometimes we will be carrying around
        // "too many tiles", or it could lead to running out of tiles if it is too short.
        // The alternative is to use `calculate_train_component_head_tails_and_final_tail_position`
        // to calculate the tail position, and then remove the last tiles if they are not needed,
        // but that introduces more complexity.
        const HEURISTIC_COEF: f32 = 2.0;
        if self.tile_path.len() > (HEURISTIC_COEF * transport_type.length_in_tiles()) as usize {
            let _ = self.tile_path.pop();
        }

        self.progress_within_tile = ProgressWithinTile::just_entering();
    }
}
