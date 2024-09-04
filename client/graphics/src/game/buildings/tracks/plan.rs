use bevy::prelude::Res;
use bevy_egui::EguiContexts;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
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
    ordered_selected_edges: &[EdgeXZ],
    ordered_selected_tiles: &[TileCoordsXZ],
    mut egui_contexts: EguiContexts,
) -> Option<Vec<TrackInfo>> {
    on_ui(&mut egui_contexts).then_none()?;

    let head_edge = ordered_selected_edges.first()?;
    let tail_edge = ordered_selected_edges.last()?;

    (head_edge == tail_edge).then_none()?;

    let head_tile = ordered_selected_tiles.first()?;
    let tail_tile = ordered_selected_tiles.last()?;

    (head_tile == tail_tile).then_none()?;

    let head = DirectionalEdge::from_tile_and_edge(*head_tile, *head_edge)?;
    let tail = DirectionalEdge::from_tile_and_edge(*tail_tile, *tail_edge)?;
    let tail = tail.mirror();

    let PlayerIdResource(player_id) = *player_id_resource;
    plan_tracks(
        player_id,
        head,
        &[tail],
        game_state,
        DEFAULT_ALREADY_EXISTS_COEF,
    )
    .map(|(track_infos, _)| track_infos)
}
