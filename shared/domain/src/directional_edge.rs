use std::fmt;
use std::fmt::{Debug, Formatter};

use shared_util::direction_xz::DirectionXZ;

use crate::edge_xz::EdgeXZ;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::tile_track::TileTrack;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct DirectionalEdge {
    pub into_tile:      TileCoordsXZ,
    pub from_direction: DirectionXZ,
}

impl Debug for DirectionalEdge {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} from {:?}", self.into_tile, self.from_direction)
    }
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
    pub fn mirror(&self) -> Self {
        let new_tile = self.into_tile + self.from_direction;
        Self::new(new_tile, self.from_direction.reverse())
    }

    #[must_use]
    pub fn entrance_to(tile_track: TileTrack) -> Self {
        Self::new(
            tile_track.tile,
            tile_track
                .track_type
                .other_end_unsafe(tile_track.pointing_in),
        )
    }

    #[must_use]
    pub fn from_tile_and_edge(tile: TileCoordsXZ, edge: EdgeXZ) -> Option<Self> {
        edge.both_tiles_and_directions()
            .into_iter()
            .find(|(tile_coords, _)| *tile_coords == tile)
            .map(|(_, direction)| Self::new(tile, direction))
    }

    #[must_use]
    pub const fn exit_from(tile_track: TileTrack) -> Self {
        Self::new(tile_track.tile, tile_track.pointing_in.reverse())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::track_type::TrackType;

    #[test]
    fn test_entrance_to() {
        let tile_track = TileTrack {
            tile:        TileCoordsXZ::new(1, 2),
            pointing_in: DirectionXZ::East,
            track_type:  TrackType::NorthEast,
        };
        let directional_edge = DirectionalEdge::entrance_to(tile_track);
        assert_eq!(
            directional_edge,
            DirectionalEdge::new(TileCoordsXZ::new(1, 2), DirectionXZ::North)
        );
    }

    #[test]
    fn test_exit_from() {
        let tile_track = TileTrack {
            tile:        TileCoordsXZ::new(1, 2),
            pointing_in: DirectionXZ::East,
            track_type:  TrackType::NorthEast,
        };
        let directional_edge = DirectionalEdge::exit_from(tile_track);
        assert_eq!(
            directional_edge,
            DirectionalEdge::new(TileCoordsXZ::new(1, 2), DirectionXZ::West)
        );
    }

    #[test]
    fn test_mirror() {
        let tile = TileCoordsXZ::new(1, 2);
        let edge = DirectionalEdge::new(tile, DirectionXZ::North);
        let mirrored = edge.mirror();
        let expected = DirectionalEdge::new(tile + DirectionXZ::North, DirectionXZ::South);
        assert_eq!(mirrored, expected);
    }
}
