use std::time::{Duration, Instant};

use log::{log, trace, warn, Level};
use pathfinding::prelude::dijkstra;
use shared_util::bool_ops::BoolOps;

use crate::building::building_state::CanBuildResponse;
use crate::building::track_info::TrackInfo;
use crate::edge_xz::EdgeXZ;
use crate::game_state::GameState;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_length::TrackLength;
use crate::transport::track_type::TrackType;
use crate::PlayerId;

pub const DEFAULT_ALREADY_EXISTS_COEF: f32 = 0.8;

#[must_use]
pub fn plan_tracks_edge_to_edge(
    player_id: PlayerId,
    head: EdgeXZ,
    tail: EdgeXZ,
    game_state: &GameState,
    already_exists_coef: f32,
) -> Option<Vec<TrackInfo>> {
    // TODO HIGH:   We should avoid having multiple heads.
    //              If we start from an edge that already has a track on one side (e.g. end of a
    //              station), we can use that track as the head.
    //              Otherwise we can use first selected tile to decide which direction to go, and
    //              use a "virtual" TileTrack on the other side that leads to that edge, in that
    //              direction. Just don't build it afterwards, remove from results.
    //              Whether to use a similar logic to cut down on tails is less clear, experiment
    //              both ways.
    //              End result is that there should be a warning logged whenever we cannot find a
    //              single head.

    let head_options = possible_tile_tracks(head, EdgeType::StartingFrom, player_id, game_state);
    let tail_options = possible_tile_tracks(tail, EdgeType::FinishingIn, player_id, game_state);
    head_options
        .into_iter()
        .filter_map(|head_option| {
            plan_tracks(
                player_id,
                head_option,
                &tail_options,
                game_state,
                already_exists_coef,
            )
        })
        .min_by_key(|(_, length)| *length)
        .map(|(tracks, _)| tracks)
}

#[derive(Copy, Clone)]
enum EdgeType {
    StartingFrom,
    FinishingIn,
}

fn possible_tile_tracks(
    edge: EdgeXZ,
    edge_type: EdgeType,
    player_id: PlayerId,
    game_state: &GameState,
) -> Vec<TileTrack> {
    let (a, b) = edge.ordered_tiles();
    let mut results = vec![];
    for tile in [a, b] {
        for track_type in TrackType::all() {
            for a in track_type.connections() {
                let b = track_type.other_end_unsafe(a);
                let comparison_direction = match edge_type {
                    EdgeType::StartingFrom => a,
                    EdgeType::FinishingIn => b,
                };
                let comparison_edge = EdgeXZ::from_tile_and_direction(tile, comparison_direction);
                if edge == comparison_edge {
                    let tile_track = TileTrack {
                        tile_coords_xz: tile,
                        track_type,
                        pointing_in: b,
                    };
                    let track_info = TrackInfo::from_tile_track(player_id, tile_track);
                    let response = game_state.can_build_track(player_id, &track_info);
                    match response {
                        CanBuildResponse::Ok | CanBuildResponse::AlreadyExists => {
                            results.push(tile_track);
                        },
                        CanBuildResponse::Invalid => {},
                    }
                }
            }
        }
    }

    results
}

fn successors(
    current_tile_track: TileTrack,
    player_id: PlayerId,
    game_state: &GameState,
    already_exists_coef: f32,
) -> Vec<(TileTrack, TrackLength)> {
    let next_tile_coords = current_tile_track.next_tile_coords();

    let mut results = vec![];
    for track_type in TrackType::all() {
        if let Some(pointing_in) = track_type.other_end(current_tile_track.pointing_in.reverse()) {
            let tile_track = TileTrack {
                tile_coords_xz: next_tile_coords,
                track_type,
                pointing_in,
            };
            let track_info = TrackInfo::from_tile_track(player_id, tile_track);
            let response = game_state.can_build_track(player_id, &track_info);
            let coef = response_to_coef(response, already_exists_coef);
            if let Some(coef) = coef {
                let adjusted_length = tile_track.track_type.length() * coef;
                results.push((tile_track, adjusted_length));
            }
        }
    }

    trace!("current: {current_tile_track:?}, successors: {results:?}");
    results
}

fn response_to_coef(can_build_response: CanBuildResponse, already_exists_coef: f32) -> Option<f32> {
    match can_build_response {
        CanBuildResponse::Ok => Some(1f32),
        CanBuildResponse::AlreadyExists => Some(already_exists_coef),
        CanBuildResponse::Invalid => None,
    }
}

#[must_use]
pub fn plan_tracks(
    player_id: PlayerId,
    current_tile_track: TileTrack,
    targets: &[TileTrack],
    game_state: &GameState,
    already_exists_coef: f32,
) -> Option<(Vec<TrackInfo>, TrackLength)> {
    targets.is_empty().then_none()?;

    let start = Instant::now();

    let path: Option<(Vec<TileTrack>, TrackLength)> = dijkstra(
        &current_tile_track,
        |tile_track| successors(*tile_track, player_id, game_state, already_exists_coef),
        |tile_track| targets.contains(tile_track),
    );

    let result = path.map(|(path, length)| {
        let mut tracks = vec![];

        for tile_track in path {
            let track_info = TrackInfo::from_tile_track(player_id, tile_track);

            match game_state.can_build_track(player_id, &track_info) {
                CanBuildResponse::Ok => {
                    tracks.push(track_info);
                },
                CanBuildResponse::AlreadyExists => {
                    // Expected if we are building an addition to existing track
                },
                CanBuildResponse::Invalid => {
                    warn!(
                        "Unexpected state - our found path includes invalid tracks: {:?}",
                        tile_track,
                    );
                },
            }
        }

        (tracks, length)
    });

    // TODO:    We could precompute using `dijkstra_all` async and then just look up the result here, possibly with some caching.
    //          See https://github.com/loopystudios/bevy_async_task
    let elapsed = start.elapsed();
    let level = if elapsed > Duration::from_millis(100) {
        Level::Warn
    } else if elapsed > Duration::from_millis(20) {
        Level::Info
    } else if elapsed > Duration::from_millis(10) {
        Level::Debug
    } else {
        Level::Trace
    };
    log!(
        level,
        "Planning tracks from {current_tile_track:?} to {targets:?} took {:?}",
        elapsed,
    );

    result
}

#[cfg(test)]
mod tests {
    use shared_util::direction_xz::DirectionXZ;

    use super::*;
    use crate::map_level::map_level::MapLevel;
    use crate::tile_coords_xz::TileCoordsXZ;
    use crate::MapId;

    #[test]
    fn test_plan_tracks_edge_to_edge() {
        let player_id = PlayerId::random();

        let mut game_state = GameState::empty_from_level(
            MapId("usa_east".to_string()),
            MapLevel::load(include_str!("../../../../assets/map_levels/usa_east.json")),
        );

        let result = plan_tracks_edge_to_edge(
            player_id,
            EdgeXZ::from_tile_and_direction(TileCoordsXZ::new(1, 190), DirectionXZ::West),
            EdgeXZ::from_tile_and_direction(TileCoordsXZ::new(255, 0), DirectionXZ::South),
            &game_state,
            DEFAULT_ALREADY_EXISTS_COEF,
        );
        match result {
            None => {
                panic!("No result");
            },
            Some(tracks) => {
                println!("{}", tracks.len());
                assert!(tracks.len() > 450);
                let result = game_state.build_tracks(player_id, &tracks);
                match result {
                    Ok(results) => assert_eq!(results.len(), tracks.len()),
                    Err(()) => panic!("Failed to build tracks"),
                }
            },
        }
    }
}
