use itertools::Itertools;
use log::{Level, info, log, trace, warn};
use pathfinding::prelude::dijkstra;
use shared_util::bool_ops::BoolOptionOps;
use web_time::{Duration, Instant};

use crate::PlayerId;
use crate::building::building_state::CanBuildResponse;
use crate::building::track_info::TrackInfo;
use crate::directional_edge::DirectionalEdge;
use crate::game_state::GameState;
use crate::metrics::Metrics;
use crate::transport::track_length::TrackLength;
use crate::transport::track_type::TrackType;

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
        let coef = response_to_coef(&response, already_exists_coef);
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

fn response_to_coef(
    can_build_response: &CanBuildResponse,
    already_exists_coef: f32,
) -> Option<f32> {
    match can_build_response {
        CanBuildResponse::Ok => Some(1f32),
        CanBuildResponse::AlreadyExists => Some(already_exists_coef),
        CanBuildResponse::Invalid(_) => None,
    }
}

#[must_use]
pub fn plan_tracks(
    player_id: PlayerId,
    current: DirectionalEdge,
    targets: &[DirectionalEdge],
    game_state: &GameState,
    already_exists_coef: f32,
    metrics: &dyn Metrics,
) -> Option<(Vec<TrackInfo>, TrackLength)> {
    targets.is_empty().then_none()?;

    let start = Instant::now();

    trace!("Planning tracks at {start:?} from {current:?} to {targets:?}");

    // TODO: Consider optimising either by `dijkstra_all` or Floyd-Warshall
    let path = dijkstra(
        &current,
        |current| successors(*current, player_id, game_state, already_exists_coef),
        |current| targets.contains(current),
    );

    let path_length = path.as_ref().map(|(path, _length)| path.len());
    info!(
        "Found path from {current:?} to {targets:?} in {:?}: {:?} length",
        start.elapsed(),
        path_length
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
                    CanBuildResponse::Invalid(error) => {
                        warn!(
                            "Unexpected state - our found path includes invalid tracks: {current:?}, {error:?}",
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
    let lengths = result
        .as_ref()
        .map(|(tracks, length)| (tracks.len(), *length));
    log!(
        level,
        "Planning tracks ({:?}) from {current:?} to {targets:?} took {:?}",
        lengths,
        elapsed,
    );
    metrics.track_planning_duration(elapsed, lengths);

    result
}

#[cfg(test)]
mod tests {
    use shared_util::direction_xz::DirectionXZ;

    use super::*;
    use crate::building::building_info::WithOwner;
    use crate::map_level::map_level::{Height, MapLevel, TerrainType};
    use crate::map_level::terrain::Terrain;
    use crate::map_level::zoning::Zoning;
    use crate::metrics::NoopMetrics;
    use crate::scenario::Scenario;
    use crate::tile_coords_xz::TileCoordsXZ;
    use crate::water::Water;
    use crate::{MapId, ScenarioId};

    #[test]
    fn test_plan_single_tile_ew() {
        let size_x = 3;
        let size_z = 3;
        let player_id = PlayerId::random();
        let tile = TileCoordsXZ::new(1, 1);
        let terrain = Terrain::flat(size_x, size_z, Height::from_u8(1), TerrainType::Grass);
        let water = Water::new(Height::from_u8(0), Height::from_u8(1));
        let zoning = Zoning::new(size_x, size_z);
        let map_level = MapLevel::new(
            MapId("test".to_string()),
            terrain,
            water.expect("valid water"),
            zoning,
        );
        let scenario = Scenario {
            scenario_id: ScenarioId("test".to_string()),
            players: vec![],
            map_level,
        };
        let game_state = GameState::from_scenario(scenario, false);
        let head = DirectionalEdge::new(tile, DirectionXZ::West);
        let tail = DirectionalEdge::new(tile + DirectionXZ::East, DirectionXZ::West);
        let (results, length) = plan_tracks(
            player_id,
            head,
            &[tail],
            &game_state,
            DEFAULT_ALREADY_EXISTS_COEF,
            &NoopMetrics::default(),
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(length, TrackType::WestEast.length());

        let track = &results[0];
        assert_eq!(track.owner_id(), player_id);
        assert_eq!(track.tile, tile);
        assert_eq!(track.track_type, TrackType::WestEast);
    }
}
