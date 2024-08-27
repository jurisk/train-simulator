use std::collections::HashSet;

use log::warn;
use pathfinding::prelude::dijkstra;

use crate::building::building_state::CanBuildResponse;
use crate::building::track_info::TrackInfo;
use crate::edge_xz::EdgeXZ;
use crate::game_state::GameState;
use crate::tile_coords_xz::TileCoordsXZ;
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
    preferred_tiles: &HashSet<TileCoordsXZ>,
    game_state: &GameState,
    player_id: PlayerId,
) -> Vec<(EdgeXZ, TrackLength)> {
    let mut results = vec![];

    for tile in edge.ordered_tiles() {
        for neighbour in EdgeXZ::for_tile(tile) {
            const NON_PREFERRED_TILE_MALUS: f32 = 16f32; // How much shorter "length" do we assign to going through a preferred tile

            for tile_track in track_types_that_fit(edge, neighbour) {
                let track = TrackInfo::new(
                    TrackId::random(),
                    player_id,
                    tile_track.tile_coords_xz,
                    tile_track.track_type,
                );

                let malus = if preferred_tiles.contains(&tile) {
                    1f32
                } else {
                    NON_PREFERRED_TILE_MALUS
                };

                // Later:
                //  - Should we give a bonus in case the track already exists?
                //  - Bonus or malus if the existing track is provided by a station?
                //  - Ignore the `preferred_tiles` altogether?

                if matches!(
                    game_state.can_build_track(player_id, &track),
                    CanBuildResponse::Ok | CanBuildResponse::AlreadyExists
                ) {
                    results.push((neighbour, tile_track.track_type.length() * malus));
                }
            }
        }
    }

    results
}

// TODO HIGH: Just ignore `ordered_selected_tiles` and `ordered_selected_edges`, it leads to confusing results
#[must_use]
pub fn plan_tracks(
    player_id: PlayerId,
    ordered_selected_tiles: &[TileCoordsXZ],
    ordered_selected_edges: &[EdgeXZ],
    game_state: &GameState,
) -> Option<Vec<TrackInfo>> {
    let head = *ordered_selected_edges.first()?;
    let tail = *ordered_selected_edges.last()?;

    let preferred_tiles: HashSet<TileCoordsXZ> = ordered_selected_tiles.iter().copied().collect();

    // Later: If `tail` is under water, no sense to plan?
    // Later: Consider switching to `a_star` or `dijkstra_all`
    let path: Option<(Vec<EdgeXZ>, TrackLength)> = dijkstra(
        &head,
        |edge| successors(*edge, &preferred_tiles, game_state, player_id),
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
