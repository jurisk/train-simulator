use std::time::{Duration, Instant};

use itertools::Itertools;
use log::{log, trace, warn, Level};
use pathfinding::prelude::dijkstra;
use shared_util::bool_ops::BoolOps;

use crate::building::building_state::CanBuildResponse;
use crate::building::track_info::TrackInfo;
use crate::directional_edge::DirectionalEdge;
use crate::game_state::GameState;
use crate::transport::track_length::TrackLength;
use crate::transport::track_type::TrackType;
use crate::PlayerId;

pub const DEFAULT_ALREADY_EXISTS_COEF: f32 = 0.8;

fn successors(
    current: DirectionalEdge,
    player_id: PlayerId,
    game_state: &GameState,
    already_exists_coef: f32,
) -> Vec<(DirectionalEdge, TrackLength)> {
    let mut results = Vec::with_capacity(3);
    let tile = current.into_tile;
    for track_type in TrackType::matching_direction(current.from_direction) {
        let response = game_state.can_build_track_internal(player_id, tile, track_type);
        let coef = response_to_coef(response, already_exists_coef);
        if let Some(coef) = coef {
            let adjusted_length = track_type.length() * coef;
            if let Some(exit_direction) = track_type.other_end(current.from_direction) {
                let next_from_direction = exit_direction.reverse();
                let next_tile = current.into_tile + exit_direction;
                let next_edge = DirectionalEdge::new(next_tile, next_from_direction);
                results.push((next_edge, adjusted_length));
            }
        }
    }
    trace!("current: {current:?}, successors: {results:?}");
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
    current: DirectionalEdge,
    targets: &[DirectionalEdge],
    game_state: &GameState,
    already_exists_coef: f32,
) -> Option<(Vec<TrackInfo>, TrackLength)> {
    targets.is_empty().then_none()?;

    let start = Instant::now();

    let path = dijkstra(
        &current,
        |current| successors(*current, player_id, game_state, already_exists_coef),
        |current| targets.contains(current),
    );

    let result = path.map(|(path, length)| {
        let mut tracks = vec![];

        for (a, b) in path.into_iter().tuple_windows() {
            if let Some(track_type) =
                TrackType::from_directions(a.from_direction, b.from_direction.reverse())
            {
                let track_info = TrackInfo::new(player_id, a.into_tile, track_type);
                let response = game_state.can_build_track(player_id, &track_info);
                match response {
                    CanBuildResponse::Ok => {
                        tracks.push(track_info);
                    },
                    CanBuildResponse::AlreadyExists => {
                        // Expected if we are building an addition to existing track
                    },
                    CanBuildResponse::Invalid => {
                        warn!(
                            "Unexpected state - our found path includes invalid tracks: {:?}",
                            current,
                        );
                    },
                }
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
        "Planning tracks ({:?}) from {current:?} to {targets:?} took {:?}",
        result
            .as_ref()
            .map(|(tracks, length)| (tracks.len(), length)),
        elapsed,
    );

    result
}

// TODO HIGH: Move to integration tests
#[cfg(test)]
mod tests {
    use shared_util::direction_xz::DirectionXZ;

    use super::*;
    use crate::map_level::map_level::MapLevel;
    use crate::tile_coords_xz::TileCoordsXZ;
    use crate::transport::tile_track::TileTrack;
    use crate::transport::track_pathfinding::find_route_to_tile_tracks;
    use crate::MapId;

    #[test]
    fn test_plan_tracks() {
        let player_id = PlayerId::random();

        let mut game_state = GameState::empty_from_level(
            MapId("usa_east".to_string()),
            MapLevel::load(include_str!("../../../../assets/map_levels/usa_east.json")),
        );

        let from_tile = TileCoordsXZ::new(1, 190);
        let to_tile = TileCoordsXZ::new(255, 0);

        let head = DirectionalEdge::new(from_tile, DirectionXZ::West);
        let tail = DirectionalEdge::new(to_tile, DirectionXZ::South);

        let (tracks, length) = plan_tracks(
            player_id,
            head,
            &[tail],
            &game_state,
            DEFAULT_ALREADY_EXISTS_COEF,
        )
        .expect("Failed to plan tracks");

        println!("{}", tracks.len());
        assert!(tracks.len() > 450);
        assert!(length > TrackLength::new(300f32));
        let result = game_state
            .build_tracks(player_id, &tracks)
            .expect("Failed to build tracks");
        assert_eq!(result.len(), tracks.len());

        let first_tile = head.into_tile;
        let last_tile = tail.into_tile + tail.from_direction;
        println!(
            "From {:?}",
            game_state.building_state().tracks_at(first_tile)
        );
        println!(
            "To   {:?}",
            game_state.building_state().tracks_at(last_tile)
        );

        let from_tile_track = TileTrack {
            tile:        first_tile,
            track_type:  TrackType::NorthWest,
            pointing_in: DirectionXZ::North,
        };

        let to_tile_track = TileTrack {
            tile:        last_tile,
            track_type:  TrackType::NorthWest,
            pointing_in: DirectionXZ::North,
        };
        let route = find_route_to_tile_tracks(
            from_tile_track,
            &[to_tile_track],
            game_state.building_state(),
        )
        .unwrap();
        assert!(route.len() > 450);
    }
}
