use bevy::prelude::{Camera, GlobalTransform, Query, Res, Vec3};
use bevy_egui::EguiContexts;
use egui::{Align2, Context, Id, Pos2};
use shared_domain::building::building_info::{WithBuildingDynamicInfo, WithTileCoverage};
use shared_domain::game_state::GameState;

use crate::game::{center_vec3, GameStateResource};

fn with_tile_coverage_label(
    id: String,
    label: String,
    with_tile_coverage: &dyn WithTileCoverage,
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let position_3d = center_vec3(with_tile_coverage, game_state.map_level());
    draw_label(position_3d, label, id, context, camera, camera_transform);
}

#[allow(clippy::needless_pass_by_value, clippy::module_name_repetitions)]
pub fn draw_labels(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
) {
    if let Some((camera_transform, camera)) =
        camera_query.iter().find(|(_, camera)| camera.is_active)
    {
        if let Some(game_state_resource) = game_state_resource {
            let GameStateResource(game_state) = game_state_resource.as_ref();

            let context = contexts.ctx_mut();

            draw_zoning_buttons(game_state, context, camera, camera_transform);
            draw_industry_labels(game_state, context, camera, camera_transform);
            draw_station_labels(game_state, context, camera, camera_transform);
            draw_transport_labels(game_state, context, camera, camera_transform);
        }
    }
}

fn draw_zoning_buttons(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let buildings = game_state.building_state();
    for zoning_info in game_state.map_level().zoning().all_zonings() {
        if buildings
            .industry_building_at(zoning_info.reference_tile())
            .is_none()
        {
            // TODO HIGH: Make these into menu buttons to build the relevant industry buildings
            with_tile_coverage_label(
                format!("{:?}", zoning_info.id()),
                format!("{:?}", zoning_info.zoning_type()),
                zoning_info,
                game_state,
                context,
                camera,
                camera_transform,
            );
        }
    }
}

fn draw_industry_labels(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    for industry_building in game_state.building_state().all_industry_buildings() {
        let id = format!("{:?}", industry_building.id());

        let label = format!(
            "{:?} {:?}",
            industry_building.industry_type(),
            industry_building.dynamic_info()
        );

        with_tile_coverage_label(
            id,
            label,
            industry_building,
            game_state,
            context,
            camera,
            camera_transform,
        );
    }
}

fn draw_station_labels(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    for station in game_state.building_state().all_stations() {
        let id = format!("{:?}", station.id());
        let label = format!("{:?} {:?}", station.station_type(), station.dynamic_info());

        with_tile_coverage_label(
            id,
            label,
            station,
            game_state,
            context,
            camera,
            camera_transform,
        );
    }
}

fn draw_transport_labels(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    for transport in game_state.transport_infos() {
        // TODO: The cargo label could actually be "I 1.0 of 2.0" or similar
        let label = transport.cargo_as_string();
        let id = format!("{:?}", transport.transport_id());
        let transport_location = transport.location();
        let transport_position_3d = transport_location.tile_path[0].progress_coordinates(
            transport_location.progress_within_tile,
            game_state.map_level().terrain(),
        );
        draw_label(
            transport_position_3d,
            label,
            id,
            context,
            camera,
            camera_transform,
        );
    }
}

// TODO HIGH: This looks ugly and often breaks. Consider using https://docs.rs/egui/latest/egui/struct.Painter.html instead? Or https://bevyengine.org/examples/2d-rendering/text2d/ or https://github.com/kulkalkul/bevy_mod_billboard?
fn draw_label(
    position: Vec3,
    label: String,
    id: String,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let label_position = project_to_screen(position, camera, camera_transform, context);

    egui::Area::new(Id::from(id))
        .fixed_pos(label_position)
        .pivot(Align2::CENTER_CENTER)
        .constrain(false)
        .show(context, |ui| {
            ui.colored_label(egui::Color32::WHITE, label);
        });
}

#[allow(clippy::let_and_return)]
fn project_to_screen(
    position: Vec3,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    context: &Context,
) -> Pos2 {
    if let Some(ndc_space_coords) = camera.world_to_ndc(camera_transform, position) {
        let screen_size = context.screen_rect().size();
        let screen_position = Pos2::new(
            (ndc_space_coords.x + 1.0) * 0.5 * screen_size.x,
            (1.0 - ndc_space_coords.y) * 0.5 * screen_size.y,
        );

        screen_position
    } else {
        Pos2::ZERO
    }
}
