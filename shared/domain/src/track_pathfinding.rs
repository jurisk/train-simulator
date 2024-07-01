use crate::building_state::BuildingState;
use crate::tile_track::TileTrack;
use crate::track_type::TrackType;
use crate::BuildingId;

// TODO HIGH:   We need to think how to handle station expansion. Can a station have multiple buildings?
//              And thus we need `StationId` for pathfinding as `target_station`?
#[must_use]
pub fn find_route_to_station(
    current_tile_track: TileTrack,
    target_station: BuildingId,
    building_state: &BuildingState,
) -> Option<Vec<TileTrack>> {
    let next_tile_coords = current_tile_track.tile_coords_xz + current_tile_track.pointing_in;
    let tracks_at_next_tile: Vec<TrackType> = building_state.track_types_at(next_tile_coords);

    let valid_tracks_at_next_tile: Vec<TrackType> = tracks_at_next_tile
        .into_iter()
        .filter(|track_type| {
            track_type
                .connections()
                .contains(&current_tile_track.pointing_in.reverse())
        })
        .collect();

    // TODO HIGH: Implement actual pathfinding

    Some(
        valid_tracks_at_next_tile
            .into_iter()
            .map(|track_type| {
                TileTrack {
                    tile_coords_xz: next_tile_coords,
                    track_type,
                    pointing_in: track_type.other_end(current_tile_track.pointing_in.reverse()),
                }
            })
            .collect::<Vec<_>>(),
    )
}
