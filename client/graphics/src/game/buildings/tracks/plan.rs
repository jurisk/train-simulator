use bevy::prelude::Res;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::metrics::NoopMetrics;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::track_planner::{plan_tracks, DEFAULT_ALREADY_EXISTS_COEF};

use crate::game::PlayerIdResource;

// TODO: If the selected first/last tile is a station, we should snap to the station's edge
// TODO: We could improve the snapping logic, e.g. by passing in multiple tail `DirectionalEdge`-s into the pathfinding logic
pub(crate) fn try_plan_tracks(
    player_id_resource: Res<PlayerIdResource>,
    game_state: &GameState,
    head: DirectionalEdge,
    tail_tile: Option<TileCoordsXZ>,
    tail_edge: Option<EdgeXZ>,
) -> Option<Vec<TrackInfo>> {
    let tail = resolve_tail(tail_tile, tail_edge)?;

    let PlayerIdResource(player_id) = *player_id_resource;
    plan_tracks(
        player_id,
        head,
        &[tail],
        game_state,
        DEFAULT_ALREADY_EXISTS_COEF,
        &NoopMetrics::default(),
    )
    .map(|(track_infos, _)| track_infos)
}

pub(crate) fn resolve_head(
    head_tile: Option<TileCoordsXZ>,
    head_edge: Option<EdgeXZ>,
) -> Option<DirectionalEdge> {
    let head_tile = head_tile?;
    let head_edge = head_edge?;

    let head = DirectionalEdge::from_tile_and_edge(head_tile, head_edge)?;

    Some(head)
}

fn resolve_tail(
    tail_tile: Option<TileCoordsXZ>,
    tail_edge: Option<EdgeXZ>,
) -> Option<DirectionalEdge> {
    let tail_tile = tail_tile?;
    let tail_edge = tail_edge?;

    let tail = DirectionalEdge::from_tile_and_edge(tail_tile, tail_edge)?;
    let tail = tail.mirror();

    Some(tail)
}

#[cfg(test)]
mod tests {
    use shared_util::direction_xz::DirectionXZ;

    use super::*;

    #[test]
    fn test_resolve_edges_for_single_tile() {
        let tile = TileCoordsXZ::new(0, 0);
        let head_edge = EdgeXZ::from_tile_and_direction(tile, DirectionXZ::West);
        let tail_edge = EdgeXZ::from_tile_and_direction(tile, DirectionXZ::North);

        let head = resolve_head(Some(tile), Some(head_edge)).unwrap();
        let tail = resolve_tail(Some(tile), Some(tail_edge)).unwrap();

        let expected_head = DirectionalEdge::new(tile, DirectionXZ::West);
        let expected_tail = DirectionalEdge::new(tile + DirectionXZ::North, DirectionXZ::South);

        assert_eq!(head, expected_head);
        assert_eq!(tail, expected_tail);
    }
}
