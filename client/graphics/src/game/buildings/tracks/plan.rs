use bevy::prelude::Res;
use bevy_egui::EguiContexts;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::transport::track_planner::{
    plan_tracks_edge_to_edge, DEFAULT_ALREADY_EXISTS_COEF,
};
use shared_util::bool_ops::BoolOps;

use crate::game::PlayerIdResource;
use crate::on_ui;

pub(crate) fn try_plan_tracks(
    player_id_resource: Res<PlayerIdResource>,
    game_state: &GameState,
    ordered_selected_edges: &[EdgeXZ],
    mut egui_contexts: EguiContexts,
) -> Option<Vec<TrackInfo>> {
    on_ui(&mut egui_contexts).then_none()?;

    let head = ordered_selected_edges.first()?;
    let tail = ordered_selected_edges.last()?;

    (head == tail).then_none()?;

    let PlayerIdResource(player_id) = *player_id_resource;
    plan_tracks_edge_to_edge(
        player_id,
        *head,
        *tail,
        game_state,
        DEFAULT_ALREADY_EXISTS_COEF,
    )
}
