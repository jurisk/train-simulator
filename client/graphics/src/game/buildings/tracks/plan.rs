use bevy::prelude::Res;
use bevy_egui::EguiContexts;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::metrics::NoopMetrics;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::track_planner::{plan_tracks, DEFAULT_ALREADY_EXISTS_COEF};
use shared_util::bool_ops::BoolOps;

use crate::game::PlayerIdResource;
use crate::on_ui;

// TODO: If the selected first/last tile is a station, we should snap to the station's edge
// TODO: We could improve the snapping logic, e.g. by passing in multiple tail `DirectionalEdge`-s into the pathfinding logic
pub(crate) fn try_plan_tracks(
    player_id_resource: Res<PlayerIdResource>,
    game_state: &GameState,
    head: (Option<TileCoordsXZ>, Option<EdgeXZ>),
    tail: (Option<TileCoordsXZ>, Option<EdgeXZ>),
    mut egui_contexts: EguiContexts,
) -> Option<Vec<TrackInfo>> {
    on_ui(&mut egui_contexts).then_none()?;

    let (head, tail) = resolve_edges(head, tail)?;

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

// Later: This is a bit clumsy as it could be resolved already in `selection` logic.
fn resolve_edges(
    head: (Option<TileCoordsXZ>, Option<EdgeXZ>),
    tail: (Option<TileCoordsXZ>, Option<EdgeXZ>),
) -> Option<(DirectionalEdge, DirectionalEdge)> {
    let (head_tile, head_edge) = head;
    let (tail_tile, tail_edge) = tail;

    let head_tile = head_tile?;
    let head_edge = head_edge?;

    let tail_tile = tail_tile?;
    let tail_edge = tail_edge?;

    (head_edge == tail_edge).then_none()?;

    let head = DirectionalEdge::from_tile_and_edge(head_tile, head_edge)?;
    let tail = DirectionalEdge::from_tile_and_edge(tail_tile, tail_edge)?;
    let tail = tail.mirror();

    Some((head, tail))
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

        let (head, tail) =
            resolve_edges((Some(tile), Some(head_edge)), (Some(tile), Some(tail_edge))).unwrap();

        let expected_head = DirectionalEdge::new(tile, DirectionXZ::West);
        let expected_tail = DirectionalEdge::new(tile + DirectionXZ::North, DirectionXZ::South);

        assert_eq!(head, expected_head);
        assert_eq!(tail, expected_tail);
    }
}
