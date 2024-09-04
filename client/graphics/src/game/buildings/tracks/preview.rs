use bevy::color::palettes::basic::BLUE;
use bevy::prelude::{Gizmos, Res};
use bevy_egui::EguiContexts;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::map_level::terrain::Terrain;

use crate::game::buildings::tracks::plan::try_plan_tracks;
use crate::game::buildings::tracks::positions::rail_positions;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::selection::SelectedEdges;

// TODO HIGH: Store the track preview in a resource so we can reuse it when building tracks
// Later: Do the planning for preview `async` using https://github.com/loopystudios/bevy_async_task
// Later: Don't instantly plan when mouse is moved rapidly, instead wait for a small delay
pub(crate) fn show_track_preview(
    selected_edges: Res<SelectedEdges>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    selected_mode_resource: Res<SelectedMode>,
    egui_contexts: EguiContexts,
    mut gizmos: Gizmos,
) {
    if selected_mode_resource.as_ref() != &SelectedMode::Tracks {
        return;
    }

    let GameStateResource(game_state) = game_state_resource.as_ref();

    let ordered_selected_edges = &selected_edges.as_ref().ordered;

    if let Some(tracks) = try_plan_tracks(
        player_id_resource,
        game_state,
        ordered_selected_edges,
        egui_contexts,
    ) {
        for track in tracks {
            debug_draw_track(track, &mut gizmos, game_state.map_level().terrain());
        }
    }
}

fn debug_draw_track(track_info: TrackInfo, gizmos: &mut Gizmos, terrain: &Terrain) {
    let ((a1, a2), (b1, b2)) = rail_positions(track_info.tile, track_info.track_type, terrain);
    let color = BLUE;

    gizmos.line(a1, b2, color);
    gizmos.line(a2, b1, color);
}
