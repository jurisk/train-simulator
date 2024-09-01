use log::{debug, trace, warn};
use pathfinding::prelude::dijkstra;

use crate::building::building_state::CanBuildResponse;
use crate::building::track_info::TrackInfo;
use crate::edge_xz::EdgeXZ;
use crate::game_state::GameState;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_length::TrackLength;
use crate::transport::track_type::TrackType;
use crate::PlayerId;

#[must_use]
pub fn plan_tracks_edge_to_edge(
    player_id: PlayerId,
    head: EdgeXZ,
    tail: EdgeXZ,
    game_state: &GameState,
) -> Option<Vec<TrackInfo>> {
    let head_options = possible_tile_tracks(head, EdgeType::StartingFrom, player_id, game_state);
    let tail_options = possible_tile_tracks(tail, EdgeType::FinishingIn, player_id, game_state);
    head_options
        .into_iter()
        .filter_map(|head_option| plan_tracks(player_id, head_option, &tail_options, game_state))
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
    let mut ok_results = vec![];
    let mut already_exists_results = vec![];
    for tile in [a, b] {
        for track_type in TrackType::all() {
            for pointing_in in track_type.connections() {
                // TODO HIGH: This is quite broken and doesn't build exactly what is requested
                let other = track_type.other_end_unsafe(pointing_in);
                let comparison_direction = match edge_type {
                    EdgeType::StartingFrom => pointing_in,
                    EdgeType::FinishingIn => other,
                };
                let comparison_edge = EdgeXZ::from_tile_and_direction(tile, comparison_direction);
                if edge == comparison_edge {
                    let tile_track = TileTrack {
                        tile_coords_xz: tile,
                        track_type,
                        pointing_in: comparison_direction,
                    };
                    let track_info = TrackInfo::from_tile_track(player_id, tile_track);
                    let response = game_state.can_build_track(player_id, &track_info);
                    match response {
                        CanBuildResponse::Ok => {
                            ok_results.push(tile_track);
                        },
                        CanBuildResponse::AlreadyExists => {
                            already_exists_results.push(tile_track);
                        },
                        CanBuildResponse::Invalid => {},
                    }
                }
            }
        }
    }

    // We prefer existing tracks over fully new ones
    if already_exists_results.is_empty() {
        ok_results
    } else {
        already_exists_results
    }
}

fn successors(
    current_tile_track: TileTrack,
    player_id: PlayerId,
    game_state: &GameState,
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
            let coef = response_to_coef(response);
            if let Some(coef) = coef {
                let adjusted_length = tile_track.track_type.length() * coef;
                results.push((tile_track, adjusted_length));
            }
        }
    }

    trace!("current: {current_tile_track:?}, successors: {results:?}");
    results
}

fn response_to_coef(can_build_response: CanBuildResponse) -> Option<f32> {
    match can_build_response {
        CanBuildResponse::Ok => Some(1f32),
        CanBuildResponse::AlreadyExists => Some(1f32 / 4f32),
        CanBuildResponse::Invalid => None,
    }
}

#[must_use]
pub fn plan_tracks(
    player_id: PlayerId,
    current_tile_track: TileTrack,
    targets: &[TileTrack],
    game_state: &GameState,
) -> Option<(Vec<TrackInfo>, TrackLength)> {
    debug!("Planning tracks for {player_id:?} from {current_tile_track:?} to {targets:?}");
    let path: Option<(Vec<TileTrack>, TrackLength)> = dijkstra(
        &current_tile_track,
        |tile_track| successors(*tile_track, player_id, game_state),
        |tile_track| targets.contains(tile_track),
    );

    path.map(|(path, length)| {
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
    })
}
