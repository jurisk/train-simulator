use log::debug;
use pathfinding::prelude::dijkstra;

use crate::building_state::BuildingState;
use crate::building_type::BuildingType;
use crate::transport::movement_orders::MovementOrderLocation;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_length::TrackLength;
use crate::transport::track_type::TrackType;

fn successors(
    tile_track: TileTrack,
    building_state: &BuildingState,
) -> Vec<(TileTrack, TrackLength)> {
    let next_tile_coords = tile_track.tile_coords_xz + tile_track.pointing_in;
    let tracks_at_next_tile: Vec<TrackType> = building_state.track_types_at(next_tile_coords);

    let valid_tracks_at_next_tile: Vec<TrackType> = tracks_at_next_tile
        .into_iter()
        .filter(|track_type| {
            track_type
                .connections()
                .contains(&tile_track.pointing_in.reverse())
        })
        .collect();

    valid_tracks_at_next_tile
        .into_iter()
        .map(|track_type| {
            let tile_track = TileTrack {
                tile_coords_xz: next_tile_coords,
                track_type,
                pointing_in: track_type.other_end(tile_track.pointing_in.reverse()),
            };
            (tile_track, track_type.length())
        })
        .collect::<Vec<_>>()
}

#[must_use]
pub fn find_location_tile_tracks(
    location: MovementOrderLocation,
    building_state: &BuildingState,
) -> Option<Vec<TileTrack>> {
    let MovementOrderLocation::StationId(station_id) = location;
    let building = building_state.find_building(station_id)?;
    let station_type = match building.building_type() {
        BuildingType::Station(station_type) => Some(station_type),
        BuildingType::Production(_) | BuildingType::Track(_) => None,
    }?;
    let targets = station_type
        .exit_tile_tracks(building.reference_tile())
        .into_iter()
        .map(|(_, _, track)| track)
        .collect::<Vec<_>>();
    Some(targets)
}

// Later:   We need to think how to handle station expansion. Can a station have multiple buildings?
//          And thus we need `StationId` for pathfinding as `target_station`?
#[must_use]
pub fn find_route_to(
    current_tile_track: TileTrack,
    go_to: MovementOrderLocation,
    building_state: &BuildingState,
) -> Option<Vec<TileTrack>> {
    let targets = find_location_tile_tracks(go_to, building_state)?;

    debug!(
        "Doing pathfinding. Current: {current_tile_track:?}, Go to: {go_to:?}, Targets: {targets:?}"
    );

    let (path, _length) = dijkstra(
        &current_tile_track,
        |tile_track| successors(*tile_track, building_state),
        |tile_track| targets.contains(tile_track),
    )?;

    debug!("Next in path is {:?}", path.get(1));

    Some(path)
}