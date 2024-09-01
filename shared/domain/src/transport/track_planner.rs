use log::warn;
use pathfinding::prelude::dijkstra;

use crate::building::building_state::CanBuildResponse;
use crate::building::track_info::TrackInfo;
use crate::edge_xz::EdgeXZ;
use crate::game_state::GameState;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_length::TrackLength;
use crate::transport::track_type::TrackType;
use crate::{PlayerId, TrackId};

// Later:   This actually allows turns that the trains cannot actually make (e.g. crossing rails),
//          so we should consider the direction of the train when planning the track.
//          Reuse the `find_route_to_station` code for train pathfinding here, except you probably have
//          to run this multiple times for various start-end `TrackTile` combos.
#[allow(
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn successors(
    edge: EdgeXZ,
    game_state: &GameState,
    player_id: PlayerId,
) -> Vec<(EdgeXZ, TrackLength)> {
    let mut results = vec![];

    for tile in edge.ordered_tiles() {
        for neighbour in EdgeXZ::for_tile(tile) {
            for tile_track in track_types_that_fit(edge, neighbour) {
                let track = TrackInfo::new(
                    TrackId::random(),
                    player_id,
                    tile_track.tile_coords_xz,
                    tile_track.track_type,
                );

                // Later:
                //  - Bonus or malus if the existing track is provided by a station?

                let result = game_state.can_build_track(player_id, &track);

                let length = tile_track.track_type.length();
                let coef = match result {
                    CanBuildResponse::Ok => Some(1f32),
                    CanBuildResponse::AlreadyExists => Some(1f32 / 4f32),
                    CanBuildResponse::Invalid => None,
                };

                if let Some(coef) = coef {
                    let adjusted_length = length * coef;
                    results.push((neighbour, adjusted_length));
                }
            }
        }
    }

    results
}

#[must_use]
pub fn plan_tracks(
    player_id: PlayerId,
    head: EdgeXZ,
    tail: EdgeXZ,
    game_state: &GameState,
) -> Option<Vec<TrackInfo>> {
    // Later: If `tail` is under water, no sense to plan?
    // Later: Consider switching to `a_star` or `dijkstra_all`
    let path: Option<(Vec<EdgeXZ>, TrackLength)> = dijkstra(
        &head,
        |edge| successors(*edge, game_state, player_id),
        |edge| *edge == tail,
    );

    path.map(|(path, _length)| {
        let mut tracks = vec![];

        for w in path.windows(2) {
            let a = w[0];
            let b = w[1];

            for tile_track in track_types_that_fit(a, b) {
                let track = TrackInfo::new(
                    TrackId::random(),
                    player_id,
                    tile_track.tile_coords_xz,
                    tile_track.track_type,
                );

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
