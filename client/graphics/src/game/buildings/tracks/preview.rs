use bevy::color::palettes::basic::{BLUE, RED};
use bevy::math::Vec3;
use bevy::prelude::{
    ButtonInput, CubicCardinalSpline, CubicGenerator, DetectChanges, Gizmos, MouseButton, Res,
    ResMut, Resource,
};
use shared_domain::building::track_info::TrackInfo;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::map_level::terrain::Terrain;

use crate::game::buildings::tracks::plan::{resolve_head, try_plan_tracks};
use crate::game::buildings::tracks::positions::rail_positions;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::{SelectedMode, TracksBuildingType};
use crate::selection::{HoveredEdge, HoveredTile};

#[derive(Eq, PartialEq)]
pub(crate) struct TrackPreview {
    pub head:   DirectionalEdge,
    pub tracks: Vec<TrackInfo>,
    pub tail:   DirectionalEdge,
}

#[derive(Resource, Default)]
pub(crate) struct TrackPreviewResource(pub Option<TrackPreview>);

impl TrackPreviewResource {
    pub fn take(&mut self) -> Option<TrackPreview> {
        std::mem::take(&mut self.0)
    }

    pub fn should_update(&self, new: Option<&TrackPreview>) -> bool {
        self.0.as_ref() != new
    }

    pub fn update(&mut self, planned: Option<TrackPreview>) {
        self.0 = planned;
    }
}

pub(crate) fn select_track_start(
    mut selected_mode_resource: ResMut<SelectedMode>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    hovered_tile: Res<HoveredTile>,
    hovered_edge: Res<HoveredEdge>,
) {
    let selected_mode = selected_mode_resource.as_ref();
    if let SelectedMode::Tracks(TracksBuildingType::SelectStart) = selected_mode {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            let directional_edge = resolve_head(hovered_tile.0, hovered_edge.0);
            if let Some(start) = directional_edge {
                *selected_mode_resource.as_mut() =
                    SelectedMode::Tracks(TracksBuildingType::SelectEnd { start });
            }
        }
    }
}

// Later: Do the planning for preview `async` using https://github.com/loopystudios/bevy_async_task
// Later: Don't instantly plan when mouse is being rapidly moved, instead wait for a small delay
pub(crate) fn update_track_preview(
    hovered_tile: Res<HoveredTile>,
    hovered_edge: Res<HoveredEdge>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    selected_mode_resource: Res<SelectedMode>,
    mut track_preview: ResMut<TrackPreviewResource>,
) {
    let selected_mode = selected_mode_resource.as_ref();
    if let SelectedMode::Tracks(TracksBuildingType::SelectEnd { start }) = selected_mode {
        let changed = hovered_tile.is_changed() || hovered_edge.is_changed();
        if changed {
            let GameStateResource(game_state) = game_state_resource.as_ref();

            let planned = try_plan_tracks(
                player_id_resource,
                game_state,
                *start,
                hovered_tile.0,
                hovered_edge.0,
            );

            if track_preview.should_update(planned.as_ref()) {
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

    if let Some(track_preview) = track_preview {
        let terrain = game_state.map_level().terrain();
        debug_draw_track_spline(track_preview, &mut gizmos, terrain);

        for track_info in &track_preview.tracks {
            debug_draw_track(track_info, &mut gizmos, terrain);
        }
    }
}

#[expect(clippy::expect_used)]
fn debug_draw_track_spline(track_preview: &TrackPreview, gizmos: &mut Gizmos, terrain: &Terrain) {
    let mut points: Vec<Vec3> = vec![terrain.directional_edge_entry_coordinate(track_preview.head)];

    let mut current_edge = track_preview.head.from_direction;
    for track_info in &track_preview.tracks {
        let next_edge = track_info.track_type.other_end_unsafe(current_edge);
        let coordinate = terrain.edge_center_coordinate(next_edge, track_info.tile);
        points.push(coordinate);
        current_edge = next_edge.reverse();
    }

    if points.len() > 3 {
        let curve = CubicCardinalSpline::new_catmull_rom(points).to_curve();
        gizmos.linestrip(curve.expect("success").iter_positions(50), RED);
    }
}

fn debug_draw_track(track_info: &TrackInfo, gizmos: &mut Gizmos, terrain: &Terrain) {
    let ((a1, a2), (b1, b2)) = rail_positions(track_info.tile, track_info.track_type, terrain);
    let color = BLUE;

    gizmos.line(a1, b2, color);
    gizmos.line(a2, b1, color);
}
