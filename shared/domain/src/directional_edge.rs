use shared_util::direction_xz::DirectionXZ;

use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::tile_track::TileTrack;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct DirectionalEdge {
    pub into_tile:      TileCoordsXZ,
    pub from_direction: DirectionXZ,
}

impl DirectionalEdge {
    #[must_use]
    pub const fn new(into_tile: TileCoordsXZ, from_direction: DirectionXZ) -> Self {
        Self {
            into_tile,
            from_direction,
        }
    }

    #[must_use]
    pub fn entrance_to(tile_track: TileTrack) -> Self {
        Self::new(
            tile_track.tile,
            tile_track
                .track_type
                .other_end_unsafe(tile_track.pointing_in)
                .reverse(),
        )
    }

    #[must_use]
    pub const fn exit_from(tile_track: TileTrack) -> Self {
        Self::new(tile_track.tile, tile_track.pointing_in.reverse())
    }
}
