use bevy::color::palettes::basic::BLUE;
use bevy::prelude::{Gizmos, Res, ResMut, Resource};
use bevy_egui::EguiContexts;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::map_level::terrain::Terrain;

use crate::game::buildings::tracks::plan::try_plan_tracks;
use crate::game::buildings::tracks::positions::rail_positions;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::selection::SelectedEdges;

#[derive(Resource, Default)]
pub(crate) struct TrackPreviewResource(pub Vec<TrackInfo>);

impl TrackPreviewResource {
    pub fn take(&mut self) -> Vec<TrackInfo> {
        std::mem::take(&mut self.0)
    }
}

// Later: Do the planning for preview `async` using https://github.com/loopystudios/bevy_async_task
// Later: Don't instantly plan when mouse is being rapidly moved, instead wait for a small delay
pub(crate) fn update_track_preview(
    selected_edges: Res<SelectedEdges>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    selected_mode_resource: Res<SelectedMode>,
    egui_contexts: EguiContexts,
    mut track_preview_resource: ResMut<TrackPreviewResource>,
) {
    let tracks = if selected_mode_resource.as_ref() == &SelectedMode::Tracks {
        let GameStateResource(game_state) = game_state_resource.as_ref();

        let ordered_selected_edges = &selected_edges.as_ref().ordered;

        try_plan_tracks(
            player_id_resource,
            game_state,
            ordered_selected_edges,
            egui_contexts,
        )
        .unwrap_or_default()
    } else {
        vec![]
    };

    let TrackPreviewResource(track_preview) = &mut *track_preview_resource;
    *track_preview = tracks;
}

pub(crate) fn draw_track_preview(
    track_preview_resource: Res<TrackPreviewResource>,
    mut gizmos: Gizmos,
    game_state_resource: Res<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_ref();
    let TrackPreviewResource(track_preview) = track_preview_resource.as_ref();

    for track_info in track_preview {
        debug_draw_track(track_info, &mut gizmos, game_state.map_level().terrain());
    }
}

fn debug_draw_track(track_info: &TrackInfo, gizmos: &mut Gizmos, terrain: &Terrain) {
    let ((a1, a2), (b1, b2)) = rail_positions(track_info.tile, track_info.track_type, terrain);
    let color = BLUE;

    gizmos.line(a1, b2, color);
    gizmos.line(a2, b1, color);
}
