use bevy::color::palettes::basic::BLUE;
use bevy::prelude::{info, DetectChanges, Gizmos, Res, ResMut, Resource};
use bevy_egui::EguiContexts;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::map_level::terrain::Terrain;

use crate::game::buildings::tracks::plan::try_plan_tracks;
use crate::game::buildings::tracks::positions::rail_positions;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::selection::{ClickedEdge, ClickedTile, HoveredEdge, HoveredTile};

#[derive(Resource, Default)]
pub(crate) struct TrackPreviewResource(pub Vec<TrackInfo>);

impl TrackPreviewResource {
    pub fn take(&mut self) -> Vec<TrackInfo> {
        info!("TrackPreviewResource::take {}", self.0.len());
        std::mem::take(&mut self.0)
    }

    pub fn should_update(&self, new: &[TrackInfo]) -> bool {
        self.0 != new
    }

    pub fn update(&mut self, planned: Vec<TrackInfo>) {
        info!("TrackPreviewResource::update {}", planned.len());
        self.0 = planned;
    }
}

// Later: Do the planning for preview `async` using https://github.com/loopystudios/bevy_async_task
// Later: Don't instantly plan when mouse is being rapidly moved, instead wait for a small delay
#[expect(clippy::too_many_arguments)]
pub(crate) fn update_track_preview(
    clicked_tile: Res<ClickedTile>,
    hovered_tile: Res<HoveredTile>,
    clicked_edge: Res<ClickedEdge>,
    hovered_edge: Res<HoveredEdge>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    selected_mode_resource: Res<SelectedMode>,
    egui_contexts: EguiContexts,
    mut track_preview: ResMut<TrackPreviewResource>,
) {
    if selected_mode_resource.as_ref() == &SelectedMode::Tracks {
        let changed = clicked_tile.is_changed()
            || clicked_edge.is_changed()
            || hovered_tile.is_changed()
            || hovered_edge.is_changed();
        if changed {
            let GameStateResource(game_state) = game_state_resource.as_ref();

            let planned = try_plan_tracks(
                player_id_resource,
                game_state,
                (hovered_tile.0, hovered_edge.0),
                (clicked_tile.0, clicked_edge.0),
                egui_contexts,
            )
            .unwrap_or_default();

            // TODO HIGH: There are some race conditions if the `clicked_*` is updated to `None` upon mouse release, and then `track_preview` is cleared, and only then we try to build the tracks.
            if track_preview.should_update(&planned) {
                track_preview.update(planned);
            }
        }
    }
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
