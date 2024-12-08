use log::debug;
use pathfinding::prelude::dijkstra;
use web_time::Instant;

use crate::building::building_state::BuildingState;
use crate::metrics::Metrics;
use crate::transport::movement_orders::MovementOrderLocation;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_length::TrackLength;

fn successors(
    tile_track: TileTrack,
    building_state: &BuildingState,
) -> impl Iterator<Item = (TileTrack, TrackLength)> {
    let next_tile_coords = tile_track.next_tile_coords();
    let needed_connection = tile_track.pointing_in.reverse();

    building_state
        .track_types_with_connection(next_tile_coords, needed_connection)
        .into_iter()
        .map(move |track_type| {
            let pointing_in = track_type.other_end_unsafe(needed_connection);
            let tile_track = TileTrack {
                tile: next_tile_coords,
                track_type,
                pointing_in,
            };
            (tile_track, track_type.length())
        })
}

#[must_use]
pub fn find_location_tile_tracks(
    location: MovementOrderLocation,
    building_state: &BuildingState,
) -> Option<Vec<TileTrack>> {
    let MovementOrderLocation::Station(station_id) = location;
    let building = building_state.find_station(station_id)?;
    let targets = building
        .station_exit_tile_tracks()
        .into_iter()
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
    metrics: &impl Metrics,
) -> Option<Vec<TileTrack>> {
    let targets = find_location_tile_tracks(go_to, building_state)?;

    debug!(
        "Doing pathfinding. Current: {current_tile_track:?}, Go to: {go_to:?}, Targets: {targets:?}"
    );

    find_route_to_tile_tracks(current_tile_track, &targets, building_state, metrics)
}

#[must_use]
pub fn find_route_to_tile_tracks(
    current_tile_track: TileTrack,
    targets: &[TileTrack],
    building_state: &BuildingState,
    metrics: &impl Metrics,
) -> Option<Vec<TileTrack>> {
    let start = Instant::now();

    debug!(
        "Finding route to {:?} from {:?}",
        targets, current_tile_track
    );

    // TODO:
    //  Consider optimising by Floyd-Warshall, which gets continuously updated as the `GameState` changes?
    //  But likely do this only after signals / collisions, to ensure it doesn't have to be redone.
    //  Note that you have:
    //      - Your own Scala version at https://github.com/jurisk/advent-of-code/blob/master/scala2/src/main/scala/jurisk/algorithms/pathfinding/FloydWarshall.scala
    //      - A Rust version at https://github.com/petgraph/petgraph/blob/master/src/algo/floyd_warshall.rs
    //      - Hints on updating the graph at https://stackoverflow.com/questions/2537870/how-to-recalculate-all-pairs-shorthest-paths-on-line-if-nodes-are-getting-remove
    let result = dijkstra(
        &current_tile_track,
        |tile_track| successors(*tile_track, building_state),
        |tile_track| targets.contains(tile_track),
    );

    let elapsed = start.elapsed();
    let lengths = result
        .as_ref()
        .map(|(tracks, length)| (tracks.len(), *length));

    metrics.track_pathfinding_duration(elapsed, lengths);

    result.map(|(tracks, _length)| tracks)
}
