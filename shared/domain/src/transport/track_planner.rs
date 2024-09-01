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

// TODO HIGH:   This actually allows turns that the trains cannot actually make (e.g. crossing rails),
//              so we should consider the direction of the train when planning the track.
//              Reuse the `plan_tracks` code also for track building, except you probably have
//              to run this multiple times for various start-end `TrackTile` combos.
#[allow(
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn edge_successors(
    edge: EdgeXZ,
    game_state: &GameState,
    player_id: PlayerId,
) -> Vec<(EdgeXZ, TrackLength)> {
    let mut results = vec![];

    for tile in edge.ordered_tiles() {
        for neighbour in EdgeXZ::for_tile(tile) {
            for tile_track in track_types_that_fit(edge, neighbour) {
                let track = TrackInfo::from_tile_track(player_id, tile_track);

                // Later:
                //  - Bonus or malus if the existing track is provided by a station?

                let response = game_state.can_build_track(player_id, &track);
                let coef = response_to_coef(response);

                if let Some(coef) = coef {
                    let length = tile_track.track_type.length();
                    let adjusted_length = length * coef;
                    results.push((neighbour, adjusted_length));
                }
            }
        }
    }

    results
}

#[must_use]
pub fn plan_tracks_edge_to_edge(
    player_id: PlayerId,
    head: EdgeXZ,
    tail: EdgeXZ,
    game_state: &GameState,
) -> Option<Vec<TrackInfo>> {
    // Later: If `tail` is under water, no sense to plan?
    // Later: Consider switching to `a_star` or `dijkstra_all`
    let path: Option<(Vec<EdgeXZ>, TrackLength)> = dijkstra(
        &head,
        |edge| edge_successors(*edge, game_state, player_id),
        |edge| *edge == tail,
    );

    path.map(|(path, _length)| {
        let mut tracks = vec![];

        for w in path.windows(2) {
            let a = w[0];
            let b = w[1];

            for tile_track in track_types_that_fit(a, b) {
                let track = TrackInfo::from_tile_track(player_id, tile_track);

                match game_state.can_build_track(player_id, &track) {
                    CanBuildResponse::Ok => {
                        tracks.push(track);
                    },
                    CanBuildResponse::AlreadyExists => {
                        // Expected if we are building an addition to existing track
                    },
                    CanBuildResponse::Invalid => {
                        warn!(
                            "Unexpected state - our found path includes invalid tracks: {:?}",
                            track,
                        );
                    },
                }
            }
        }

        tracks
    })
}

fn track_types_that_fit(a: EdgeXZ, b: EdgeXZ) -> Vec<TileTrack> {
    EdgeXZ::common_tile(a, b)
        .into_iter()
        .flat_map(|tile| {
            TrackType::all()
                .into_iter()
                .flat_map(|track_type| {
                    let (da, db) = track_type.connections_clockwise();
                    let ea = EdgeXZ::from_tile_and_direction(tile, da);
                    let eb = EdgeXZ::from_tile_and_direction(tile, db);
                    // This track fits!
                    if (ea == a && eb == b) || (ea == b && eb == a) {
                        let tile_track = TileTrack {
                            tile_coords_xz: tile,
                            track_type,
                            pointing_in: db,
                        };
                        vec![tile_track]
                    } else {
                        vec![]
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
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
) -> Option<Vec<TileTrack>> {
    debug!("Planning tracks for {player_id:?} from {current_tile_track:?} to {targets:?}");
    let path: Option<(Vec<TileTrack>, TrackLength)> = dijkstra(
        &current_tile_track,
        |tile_track| successors(*tile_track, player_id, game_state),
        |tile_track| targets.contains(tile_track),
    );

    path.map(|(path, _length)| {
        let mut tracks = vec![];

        for tile_track in path {
            let track = TrackInfo::from_tile_track(player_id, tile_track);

            match game_state.can_build_track(player_id, &track) {
                CanBuildResponse::Ok => {
                    tracks.push(tile_track);
                },
                CanBuildResponse::AlreadyExists => {
                    // Expected if we are building an addition to existing track
                },
                CanBuildResponse::Invalid => {
                    warn!(
                        "Unexpected state - our found path includes invalid tracks: {:?}",
                        track,
                    );
                },
            }
        }

        tracks
    })
}
